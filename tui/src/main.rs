use std::{
    borrow::BorrowMut,
    default,
    error::Error,
    io::{stdout, Stdout},
    marker::PhantomData,
    ops::ControlFlow,
    pin::Pin,
    slice::Chunks,
};

use ratatui::{
    backend::Backend,
    prelude::*,
    widgets::{
        block::{Position, Title},
        canvas::{Canvas, MapResolution},
        Block, BorderType, Borders, Padding, Paragraph, Tabs, Widget, Wrap,
    },
};
mod drawable;
use crossterm::{
    event::{self},
    event::{poll, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use drawable::Drawable;
use std::io;
use std::sync::Arc;
pub mod maps;
pub mod ui;
use ratatui::widgets::canvas::Shape;
use ui::{Card, Hand, Ui, UiElement};

use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};

use crate::maps::boomerang_australia::BoomerangAustralia;
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

type TuiCard = u8;

struct DefaultTuiMap<M: maps::Map> {
    map: M,
    title: String,
}

impl<M: maps::Map> DefaultTuiMap<M> {
    fn new() -> Self {
        Self {
            map: M::default(),
            title: "Game".to_owned(),
        }
    }
}
impl<M: maps::Map + ratatui::widgets::canvas::Shape> TuiPage for DefaultTuiMap<M> {
    fn get_title(&mut self) -> &str {
        &self.title
    }
    fn set_title(&mut self, title: String) {
        self.title = title
    }
    fn draw<B: Backend>(&mut self, frame: &mut Frame<B>, block: Rect) {
        let canvas = Canvas::default()
            .x_bounds([0.0, M::WIDTH as f64])
            .y_bounds([0.0, M::HEIGHT as f64])
            .paint(|context| {
                self.map.render(context);
            })
            .marker(Marker::Dot);

        frame.render_widget(canvas, block);
    }
}

struct Paginate<MainPage: TuiPage, MapPage: TuiPage>(MainPage, MapPage, usize);
impl<MainPage: TuiPage, MapPage: TuiPage> Paginate<MainPage, MapPage> {
    fn increment(&mut self) {
        self.2 = match self.2 {
            0 => 1,
            1 => 0,
            _ => unreachable!(),
        }
    }
    fn decrement(&mut self) {
        self.2 = match self.2 {
            0 => 1,
            1 => 0,
            _ => unreachable!(),
        }
    }
    fn new(mainpage: MainPage, mappage: MapPage) -> Self {
        Self(mainpage, mappage, 0)
    }
    fn draw<B: Backend>(&mut self, frame: &mut Frame<B>, pagination_area: Rect, block: Rect) {
        // We need to draw either
        let titles = [self.0.get_title(), self.1.get_title()].to_vec();
        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title("Tabs"))
            .select(self.2)
            .style(Style::default().cyan())
            .highlight_style(Style::default().bold().on_black());

        frame.render_widget(tabs, pagination_area);

        match self.2 {
            0 => self.0.draw(frame, block),
            1 => self.1.draw(frame, block),
            _ => unreachable!(),
        };
    }
}
trait TuiPage {
    fn draw<B: Backend>(&mut self, frame: &mut Frame<B>, block: Rect);
    fn set_title(&mut self, title: String);
    fn get_title(&mut self) -> &str;
}

struct DefaultMainPage<H: Hand<TuiCard>> {
    hand: H,
    title: String,
}
impl<H: Hand<TuiCard>> DefaultMainPage<H> {
    fn new() -> Self {
        Self {
            hand: H::new(),
            title: "Game".to_owned(),
        }
    }
}

impl<H: Hand<TuiCard>> TuiPage for DefaultMainPage<H> {
    fn draw<B: Backend>(&mut self, frame: &mut Frame<B>, block: Rect) {}
    fn set_title(&mut self, title: String) {
        self.title = title
    }
    fn get_title(&mut self) -> &str {
        &self.title
    }
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
                    Constraint::Percentage(20),  // Paginate area
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
impl UiElement for TuiCard {
    fn new() -> Self {
        todo!()
    }
}

impl Card for TuiCard {}

struct TuiHand {
    cards: Vec<TuiCard>,
}

impl UiElement for TuiHand {
    fn new() -> Self {
        Self { cards: Vec::new() }
    }
}

impl ui::Hand<TuiCard> for TuiHand {
    fn get<const COUNT: usize>(&self, start: usize) -> &[u8] {
        let min = match (start + COUNT) > self.cards.len() {
            true => start + COUNT,
            false => self.cards.len(),
        };
        &self.cards[start..min]
    }
}
type TuiDefaults = Tui<DefaultMainPage<TuiHand>, DefaultTuiMap<BoomerangAustralia>>;
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

fn deck<B: Backend>(frame: &mut Frame<B>, rect: Rect) {
    let chunks: Vec<Rect> = Layout::default()
        .direction(Direction::Horizontal)
        .margin(4)
        .constraints([Constraint::Percentage(10); 10])
        .split(rect)
        .to_vec();
    for (idx, chunk) in chunks.iter().enumerate() {
        let block = Block::default()
            .title(format!("card {:?},{:?}", idx, chunks.len()))
            .borders(Borders::ALL);
        frame.render_widget(block, *chunk);
    }
}

struct Deck {}

fn ui<B: Backend>(frame: &mut Frame<B>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Max(1),         // Title area
                Constraint::Percentage(22), // Deck and map
                Constraint::Percentage(20), // Queries and input
            ]
            .as_ref(),
        )
        .split(frame.size());
    let block = Block::default().title("Boomerang").borders(Borders::NONE);
    frame.render_widget(block, chunks[0]);
    let block = Block::default().title("Deck").borders(Borders::ALL);
    frame.render_widget(block, chunks[1]);
    deck(frame, chunks[1]);
    let block = Block::default().title("Input").borders(Borders::ALL);
    frame.render_widget(block, chunks[2]);
}

fn render_objects<B: Backend>(
    frame: &mut Frame<B>,
    area: Rect,
    objects: Vec<Box<dyn Drawable<B>>>,
) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(1), Constraint::Min(0)])
        .split(area);
    let main_areas: Vec<Vec<Rect>> = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Max(4); 9])
        .split(layout[1])
        .iter()
        .map(|&area| {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area)
                .to_vec()
        })
        .collect();
    for (idx, object) in objects.iter().enumerate() {
        object.as_ref().draw(frame, main_areas[0][idx])
    }
}

struct Tmp {}

impl<B: Backend> Drawable<B> for Tmp {
    fn draw(&self, frame: &mut Frame<B>, area: Rect) {
        let block = Block::new()
            .borders(Borders::ALL)
            .title(format!("Some card!"));
        frame.render_widget(block, area);
    }
}
