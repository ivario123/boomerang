use std::time::Duration;

use super::{Tui, TuiPage};
use crossterm::event::{self, Event, KeyCode};
use tokio::sync::mpsc;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Controls {
    Up,
    Left,
    Right,
    Down,
    Enter,
    Tab,
    Exit,
}

impl<MainPage: TuiPage, MapPage: TuiPage> Tui<MainPage, MapPage> {
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
