pub mod paginate;
pub mod popup;
pub mod show_page;

use std::{
    error::Error,
    io::{stdout, Stdout},
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::info;
use ratatui::{
    prelude::{Backend, Constraint, CrosstermBackend, Direction, Layout, Rect},
    widgets::{Block, Borders},
    Frame,
};
use tokio::sync::{broadcast, mpsc, RwLock};

use crate::ui::{self, UiMessage};
use controls::*;

use self::{paginate::Paginate, popup::Popup, show_page::ShowPage as ShowPageTrait};

// These type aliases are used to make the code more readable by reducing repetition of the generic
// types. They are not necessary for the functionality of the code.
type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;

pub mod controls;

pub trait TuiPage: EventApi {
    fn draw<B: Backend>(&mut self, frame: &mut Frame<B>, block: Rect);
    fn set_title(&mut self, title: String);
    fn get_title(&self) -> &str;
}

#[async_trait]
pub trait TuiMonitor<Message: UiMessage + Send + 'static, InfoPopup: Popup, QueryPopup: Popup> {
    async fn select(page: Arc<RwLock<Box<Self>>>, popup: QueryPopup);
    async fn info(page: Arc<RwLock<Box<Self>>>, popup: InfoPopup);

    async fn monitor(
        page: Arc<RwLock<Box<Self>>>,
        channel: broadcast::Receiver<Message>,
        mut transmit: broadcast::Sender<Message>,
    );
    fn subscribe(
        page: Arc<RwLock<Box<Self>>>,
        channel: broadcast::Receiver<Message>,
        transmit: broadcast::Sender<Message>,
    ) where
        Self: Send + Sync + 'static,
    {
        tokio::spawn(async move { Self::monitor(page, channel, transmit).await });
    }
}
#[derive(Debug)]
pub enum TuiError {
    PopupAlreadyShowing,
}

pub struct Tui<
    MainPage: TuiPage,
    MapPage: TuiPage,
    ShowPage: ShowPageTrait,
    InfoPopup: Popup,
    QueryPopup: Popup,
    EndScreen: Popup,
> {
    terminal: Terminal,
    paginate: Paginate<MainPage, MapPage, ShowPage>,
    show_popup: bool,
    info: Option<InfoPopup>,
    query: Option<QueryPopup>,
    end_screen: Option<EndScreen>,
}

impl<
        MainPage: TuiPage,
        MapPage: TuiPage,
        ShowPage: ShowPageTrait,
        InfoPopup: Popup,
        QueryPopup: Popup,
        EndScreen: Popup,
    > Tui<MainPage, MapPage, ShowPage, InfoPopup, QueryPopup, EndScreen>
{
    pub fn show_info(&mut self, info: InfoPopup) -> Result<(), TuiError> {
        if self.show_popup {
            return Err(TuiError::PopupAlreadyShowing);
        }
        self.show_popup = true;
        self.info = Some(info);
        self.query = None;
        Ok(())
    }
    pub fn show_query(&mut self, query: QueryPopup) -> Result<(), TuiError> {
        if self.show_popup {
            if let Some(_) = self.query {
                return Err(TuiError::PopupAlreadyShowing);
            }
        }
        self.show_popup = true;
        self.query = Some(query);
        self.info = None;
        Ok(())
    }
    pub fn cleanup_popup(&mut self) {
        match &mut self.query {
            Some(page) => {
                page.exit();
            }
            None => {}
        }
        match &mut self.info {
            Some(page) => {
                page.exit();
            }
            None => {}
        }
    }
    pub fn clear_popup(&mut self) {
        self.show_popup = false;
        self.query = None;
        self.info = None;
    }
    pub fn final_result(&mut self, screen: EndScreen) {
        self.end_screen = Some(screen);
    }
}

impl<
        MainPage: TuiPage,
        MapPage: TuiPage,
        ShowPage: ShowPageTrait,
        InfoPopup: Popup,
        QueryPopup: Popup,
        EndScreen: Popup,
    > Tui<MainPage, MapPage, ShowPage, InfoPopup, QueryPopup, EndScreen>
{
    pub fn paginate(&mut self) -> &mut Paginate<MainPage, MapPage, ShowPage> {
        &mut self.paginate
    }
}

impl<
        MainPage: TuiPage,
        MapPage: TuiPage,
        ShowPage: ShowPageTrait,
        InfoPopup: Popup,
        QueryPopup: Popup,
        EndScreen: Popup,
    > Tui<MainPage, MapPage, ShowPage, InfoPopup, QueryPopup, EndScreen>
{
    // Sets up the terminal to use the crossterm backend
    fn setup_terminal() -> Result<Terminal, Box<dyn Error>> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(terminal)
    }

    fn cleanup_terminal(&mut self) {
        disable_raw_mode().unwrap();
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen).unwrap();
    }
    pub fn show_popup<B: Backend, P: Popup>(frame: &mut Frame<B>, popup: &mut P) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(100), // Paginate area
                ]
                .as_ref(),
            )
            .split(frame.size());
        popup.draw(frame, chunks[0]);
    }

    pub fn showing_popup(&self) -> bool {
        self.show_popup
    }

    fn draw<B: Backend>(
        frame: &mut Frame<B>,
        paginate: &mut Paginate<MainPage, MapPage, ShowPage>,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(10), // Paginate area
                    Constraint::Percentage(80), // Page area
                    Constraint::Percentage(10), // Controls area
                ]
                .as_ref(),
            )
            .split(frame.size());

        // Draw the pages
        paginate.draw(frame, chunks[0], chunks[1]);

        let controls = Block::default().title("Controls").borders(Borders::all());
        Controls::render(frame, chunks[2]);
        frame.render_widget(controls, chunks[2]);
    }

    /// Draws the UI in the terminal
    fn term_draw(&mut self) {
        let term = &mut self.terminal;
        term.draw(|frame| {
            Self::draw(frame, &mut self.paginate);

            match &mut self.end_screen {
                Some(screen) => Self::show_popup(frame, screen),
                _ => {
                    Self::draw(frame, &mut self.paginate);
                    if self.show_popup {
                        match &mut self.info {
                            Some(popup) => {
                                Self::show_popup(frame, popup);
                                return;
                            }
                            None => {}
                        }
                        match &mut self.query {
                            Some(popup) => {
                                Self::show_popup(frame, popup);
                                return;
                            }
                            None => {}
                        }
                    }
                }
            }
        })
        .unwrap();
    }
    pub fn main_page(&mut self) -> &mut MainPage {
        self.paginate.main_page()
    }
}
impl<
        StartPage: TuiPage + Send + 'static,
        MapPage: TuiPage + Send + 'static,
        ShowPage: ShowPageTrait + Send + 'static,
        InfoPopup: Popup + Send + 'static,
        QueryPopup: Popup + Send + 'static,
        EndScreen: Popup + Send + 'static,
    > Tui<StartPage, MapPage, ShowPage, InfoPopup, QueryPopup, EndScreen>
{
    pub fn init(mainpage: StartPage, map_page: MapPage) -> RwLock<Box<Self>> {
        let ret: Self = Self {
            paginate: Paginate::new(mainpage, map_page),
            terminal: Self::setup_terminal().unwrap(),
            show_popup: false,
            query: None,
            info: None,
            end_screen: None,
        };
        RwLock::new(Box::new(ret))
    }
}

#[async_trait::async_trait]
impl<
        StartPage: TuiPage + Send + Sync + 'static,
        MapPage: TuiPage + Send + Sync + 'static,
        ShowPage: ShowPageTrait + Send + Sync + 'static,
        InfoPopup: Popup + Send + Sync + 'static,
        QueryPopup: Popup + Send + Sync + 'static,
        EndScreen: Popup + Send + Sync + 'static,
    > ui::Ui for Tui<StartPage, MapPage, ShowPage, InfoPopup, QueryPopup, EndScreen>
{
    async fn start(ui: Arc<RwLock<Box<Self>>>)
    where
        Arc<RwLock<Box<Self>>>: Send,
    {
        // Create channels
        let (tx, mut rx) = mpsc::channel::<Controls>(32);
        let (kill_sender, kill_receiver) = mpsc::channel::<()>(32);

        // Start logging inputs
        let _ = tokio::spawn(Self::handle_inputs(tx, kill_receiver));

        // Make this event driven either by backend or user

        loop {
            // Check for user input
            {
                match rx.try_recv() {
                    Ok(Controls::Exit) => {
                        let ui_locked = ui.write().await.cleanup_terminal();

                        // Kill logging instance
                        kill_sender.send(()).await.unwrap();
                        return;
                    }
                    Ok(control) => {
                        let mut ui_write = ui.write().await;
                        info!(
                            "Controls are going to popup : {:?}",
                            ui_write.showing_popup()
                        );
                        match ui_write.showing_popup() {
                            true => {
                                match &mut ui_write.info {
                                    Some(popup) => {
                                        popup.handle_input(control);
                                        continue;
                                    }
                                    None => {}
                                }
                                match &mut ui_write.query {
                                    Some(popup) => {
                                        popup.handle_input(control);
                                        continue;
                                    }
                                    None => {}
                                }
                            }
                            false => ui_write.paginate.handle_input(control),
                        }
                    }
                    Err(_) => {}
                }
            }

            // Redraw every second
            let _ = tokio::time::sleep(Duration::from_millis(50)).await;
            let mut ui_lock = ui.write().await;
            ui_lock.term_draw();
        }
    }
}
