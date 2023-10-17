pub mod paginate;

use std::{
    cell::RefCell,
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
use log::{info, warn};
use ratatui::{
    prelude::{Backend, Constraint, CrosstermBackend, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tokio::sync::{broadcast, mpsc, Mutex, RwLock};

use crate::ui::{self, Card, UiMessage};
use controls::*;

use self::paginate::Paginate;

// These type aliases are used to make the code more readable by reducing repetition of the generic
// types. They are not necessary for the functionality of the code.
type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;
type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub mod controls;

pub trait TuiPage: EventApi {
    fn draw<B: Backend>(&mut self, frame: &mut Frame<B>, block: Rect);
    fn set_title(&mut self, title: String);
    fn get_title(&mut self) -> &str;
}

#[async_trait]
pub trait TuiMonitor<Message: UiMessage + Send + 'static> {
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

pub struct Tui<MainPage: TuiPage, MapPage: TuiPage> {
    terminal: Terminal,
    paginate: Paginate<MainPage, MapPage>,
}

impl<MainPage: TuiPage, MapPage: TuiPage> Tui<MainPage, MapPage> {
    // Sets up the terminal to use the crossterm backend
    fn setup_terminal() -> Result<Terminal> {
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

    fn draw<B: Backend>(frame: &mut Frame<B>, paginate: &mut Paginate<MainPage, MapPage>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(5),  // Paginate area
                    Constraint::Percentage(80), // Page area
                    Constraint::Percentage(5),  // Controls area
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
        term.draw(|frame| Self::draw(frame, &mut self.paginate))
            .unwrap();
    }
    pub fn main_page(&mut self) -> &mut MainPage {
        self.paginate.main_page()
    }
}
impl<StartPage: TuiPage + Send + 'static, MapPage: TuiPage + Send + 'static>
    Tui<StartPage, MapPage>
{
    pub fn init(mainpage: StartPage, mappage: MapPage) -> RwLock<Box<Self>> {
        let ret: Tui<StartPage, MapPage> = Self {
            paginate: Paginate::new(mainpage, mappage),
            terminal: Self::setup_terminal().unwrap(),
        };
        RwLock::new(Box::new(ret))
    }
}

#[async_trait::async_trait]
impl<StartPage: TuiPage + Send + Sync + 'static, MapPage: TuiPage + Send + Sync + 'static> ui::Ui
    for Tui<StartPage, MapPage>
{
    async fn start(ui: Arc<RwLock<Box<Self>>>)
    where
    Arc<RwLock<Box<Self>>>: Send,
    {
        // Create channels
        let (tx, mut rx) = mpsc::channel::<Controls>(32);
        let (kill_sender, kill_reciver) = mpsc::channel::<()>(32);

        // Start logging inputs
        let _ = tokio::spawn(Self::handle_inputs(tx, kill_reciver));

        // Make this event driven either by backend or user

        loop {
            // Check for user input
            {
                match rx.try_recv() {
                    Ok(Controls::Exit) => {
                        ui.write().await.cleanup_terminal();

                        // Kill logging instance
                        kill_sender.send(()).await.unwrap();
                        return;
                    }
                    Ok(control) => ui.write().await.paginate.handle_input(control),
                    Err(_) => {}
                }
            }

            // Redraw every second
            let _ = tokio::time::sleep(Duration::from_millis(50)).await;
            warn!("Waiting for lock on terminal");
            let mut ui_lock = ui.write().await;
            info!("Aquired the lock");
            ui_lock.term_draw();
            info!("Terminal done drawing");
        }
    }
}
