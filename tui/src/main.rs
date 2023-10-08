use std::{error::Error, sync::Arc, time::Duration};

use crate::{tui::default::mappage::DefaultTuiMap, ui::Ui};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use maps::austrailia::Australia;
use ratatui::{
    prelude::{Backend, Constraint, CrosstermBackend, Direction, Layout},
    Frame,
};
use std::io::{stdout, Stdout};
use tokio::sync::{mpsc, Mutex};
use tui::{
    default::{card::TuiHand, mainpage::DefaultMainPage, paginate::Paginate},
    TuiPage,
};

pub mod drawable;
pub mod maps;
pub mod tui;
pub mod ui;

// These type aliases are used to make the code more readable by reducing repetition of the generic
// types. They are not necessary for the functionality of the code.
type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;
type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Inputs {
    Up,
    Left,
    Right,
    Down,
    Enter,
    Tab,
    ShiftTab,
    Q,
}

struct Tui<MainPage: TuiPage, MapPage: TuiPage> {
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
                    Constraint::Percentage(5),   // Paginate area
                    Constraint::Percentage(100), // Page area
                ]
                .as_ref(),
            )
            .split(frame.size());

        // Draw the pages
        paginate.draw(frame, chunks[0], chunks[1]);
    }

    /// Draws the UI in the terminal
    fn term_draw(&mut self) {
        let term = &mut self.terminal;
        term.draw(|frame| Self::draw(frame, &mut self.paginate))
            .unwrap();
    }

    async fn handle_inputs(sender: mpsc::Sender<Inputs>, mut close: mpsc::Receiver<()>) {
        loop {
            match close.try_recv() {
                Ok(_) => return,
                Err(mpsc::error::TryRecvError::Disconnected) => return,
                _ => {}
            }
            if event::poll(Duration::from_millis(100)).unwrap() {
                if let Event::Key(key) = event::read().unwrap() {
                    match key.code {
                        KeyCode::Enter => sender.send(Inputs::Enter).await.unwrap(),
                        KeyCode::Left => sender.send(Inputs::Left).await.unwrap(),
                        KeyCode::Right => sender.send(Inputs::Right).await.unwrap(),
                        KeyCode::Up => sender.send(Inputs::Up).await.unwrap(),
                        KeyCode::Down => sender.send(Inputs::Down).await.unwrap(),
                        KeyCode::Char('q') => sender.send(Inputs::Q).await.unwrap(),
                        KeyCode::Tab => sender.send(Inputs::Tab).await.unwrap(),
                        _ => {}
                    }
                }
            }
        }
    }
}
impl<StartPage: TuiPage + Send + 'static, MapPage: TuiPage + Send + 'static>
    Tui<StartPage, MapPage>
{
    fn init(mainpage: StartPage, mappage: MapPage) -> Mutex<Box<Self>> {
        let ret = Self {
            paginate: Paginate::new(mainpage, mappage),
            terminal: Self::setup_terminal().unwrap(),
        };
        Mutex::new(Box::new(ret))
    }
}

#[async_trait::async_trait]
impl<StartPage: TuiPage + Send + 'static, MapPage: TuiPage + Send + 'static> ui::Ui
    for Tui<StartPage, MapPage>
{
    async fn start(ui: Arc<Mutex<Box<Self>>>) {
        // Create channels
        let (tx, mut rx) = mpsc::channel::<Inputs>(32);
        let (kill_sender, kill_reciver) = mpsc::channel::<()>(32);

        // Start logging inputs
        let _ = tokio::spawn(Self::handle_inputs(tx, kill_reciver));

        // Make this event driven either by backend or user

        loop {
            // Check for user input
            match rx.try_recv() {
                Ok(Inputs::Q) => {
                    // Await lock on UI object.
                    ui.lock().await.cleanup_terminal();

                    // Kill logging instance
                    kill_sender.send(()).await.unwrap();
                    return;
                }
                Ok(Inputs::Tab) => {
                    ui.lock().await.paginate.increment();
                }
                Ok(msg) => {}
                Err(_) => {}
            }

            // Redraw every second
            let _ = tokio::time::sleep(Duration::from_millis(50)).await;

            ui.lock().await.term_draw();
        }
    }
    // UI implementations
}

type TuiDefaults = Tui<DefaultMainPage<TuiHand>, tui::default::mappage::DefaultTuiMap<Australia>>;
#[tokio::main]
async fn main() {
    let mainpage = DefaultMainPage::new();
    let mappage = DefaultTuiMap::new();

    let ui: Mutex<Box<TuiDefaults>> = TuiDefaults::init(mainpage, mappage);
    let _ = tokio::spawn(async {
        TuiDefaults::start(Arc::new(ui)).await;
    })
    .await;
    println!("Shutting down");
}
