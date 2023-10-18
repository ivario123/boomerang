use std::time::Duration;

use super::{popup::Popup, Tui, TuiPage};
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    prelude::{Backend, Constraint, Direction, Layout, Rect},
    style::Stylize,
    widgets::{Paragraph, Wrap},
    Frame,
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tokio::sync::mpsc;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, EnumIter)]
pub enum Controls {
    Up,
    Left,
    Right,
    Down,
    Enter,
    Tab,
    Exit,
}

impl Into<char> for Controls {
    fn into(self) -> char {
        match self {
            Controls::Up => '\u{02191}',
            Controls::Left => '\u{02190}',
            Controls::Right => '\u{02192}',
            Controls::Down => '\u{02193}',
            Controls::Enter => '\u{23CE}',
            Controls::Tab => '\u{F523}',
            Controls::Exit => 'q',
        }
    }
}
impl Controls {
    fn explain(&self) -> String {
        match self {
            Controls::Up => "Changes focused item",
            Controls::Left => "Moves card slider",
            Controls::Right => "Moves the card slider",
            Controls::Down => "Changes focused item",
            Controls::Enter => "Performs an action",
            Controls::Tab => "Changes tab",
            Controls::Exit => "Exists out of the game",
        }
        .to_owned()
    }
}

impl Controls {
    pub fn render<B: Backend>(frame: &mut Frame<B>, block: Rect) {
        let mut constraints = Vec::new();
        let len = Self::iter().len();
        for _ in Self::iter() {
            constraints.extend([Constraint::Percentage(100 / (2 * len) as u16); 2]);
        }
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints(constraints)
            .split(block);
        for (idx, el) in Self::iter().enumerate() {
            let internal_layout: std::rc::Rc<[Rect]> = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Percentage(15), Constraint::Percentage(100)])
                .split(layout[idx * 2]);

            let explination = el.explain();

            let c: char = el.into();
            let str = c.to_string();
            let text = Paragraph::new(str.yellow());
            frame.render_widget(text, internal_layout[0]);

            frame.render_widget(
                Paragraph::new(explination).wrap(Wrap { trim: false }),
                internal_layout[1],
            );
        }
    }
}

pub trait EventApi {
    fn handle_input(&mut self, control: Controls);
}

impl<
        StartPage: TuiPage + Send + 'static,
        MapPage: TuiPage + Send + 'static,
        InfoPopup: Popup + Send + 'static,
        QueryPopup: Popup + Send + 'static,
    > Tui<StartPage, MapPage, InfoPopup, QueryPopup>
{
    /// Manages user input
    ///
    ///
    pub async fn handle_inputs(sender: mpsc::Sender<Controls>, mut close: mpsc::Receiver<()>) {
        loop {
            match close.try_recv() {
                Ok(_) => return,
                Err(mpsc::error::TryRecvError::Disconnected) => return,
                _ => {}
            }
            if event::poll(Duration::from_millis(100)).unwrap() {
                if let Event::Key(key) = event::read().unwrap() {
                    let ret = match key.code {
                        KeyCode::Enter => Some(sender.send(Controls::Enter)),
                        KeyCode::Left => Some(sender.send(Controls::Left)),
                        KeyCode::Right => Some(sender.send(Controls::Right)),
                        KeyCode::Up => Some(sender.send(Controls::Up)),
                        KeyCode::Down => Some(sender.send(Controls::Down)),
                        KeyCode::Char('q') => Some(sender.send(Controls::Exit)),
                        KeyCode::Tab => Some(sender.send(Controls::Tab)),
                        _ => None,
                    };
                    match ret {
                        Some(future) => {
                            match future.await {
                                Ok(_) => {}
                                Err(_) => {
                                    // This can only happen if the
                                    // receiver has been disconnected
                                    // the most reasonable action in this case
                                    // would be to close the input handle.
                                    close.close();
                                    return;
                                }
                            }
                        }
                        None => {}
                    }
                }
            }
        }
    }
}
