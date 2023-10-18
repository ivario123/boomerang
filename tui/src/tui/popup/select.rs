use log::info;
use ratatui::{
    prelude::{Alignment, Constraint, Direction, Layout},
    style::Stylize,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use tokio::sync::broadcast;

use crate::tui::{
    controls::{Controls, EventApi},
    TuiPage,
};

use super::{Message, Popup};

#[derive(Debug)]
pub struct Select {
    channel: broadcast::Sender<Message>,
    title: String,
    options: Vec<String>,
    selected: usize,
    width: u16,
    height: u16,
}

impl Select {
    pub fn new(
        channel: broadcast::Sender<Message>,
        options: Vec<String>,
        title: String,
        height: u16,
        width: u16,
    ) -> Self {
        Self {
            channel,
            options,
            title,
            selected: 0,
            width,
            height,
        }
    }
}

impl Popup for Select {
    fn subscribe(&mut self) -> broadcast::Receiver<Message> {
        self.channel.subscribe()
    }
    fn exit(&mut self) {
        let _ = self.channel.send(Message::Close);
    }
}
impl TuiPage for Select {
    fn draw<B: ratatui::prelude::Backend>(
        &mut self,
        frame: &mut ratatui::Frame<B>,
        _block: ratatui::prelude::Rect,
    ) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.title.clone());
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - self.height) / 2),
                Constraint::Percentage(self.height),
                Constraint::Percentage((100 - self.height) / 2),
            ])
            .split(frame.size());

        let area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - self.width) / 2),
                Constraint::Percentage(self.width),
                Constraint::Percentage((100 - self.width) / 2),
            ])
            .split(popup_layout[1])[1];
        let internal = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - 20) / 2),
                Constraint::Percentage(40),
                Constraint::Percentage((100 - 20) / 2),
            ])
            .split(area)[1];

        let mut options = Vec::new();
        for _ in &self.options {
            options.push(Constraint::Percentage(
                (100 / self.options.len()).try_into().unwrap(),
            ))
        }
        let slots = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(options.as_slice())
            .split(internal);
        frame.render_widget(Clear, area); //this clears out the background
        frame.render_widget(block, area);

        for (idx, (name, area)) in self.options.iter().zip(slots.iter()).enumerate() {
            let paragraph = Paragraph::new(match idx == self.selected {
                true => name.yellow(),
                false => name.gray(),
            })
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });
            frame.render_widget(paragraph, *area);
        }

        info!("Popup drawn!");
    }

    fn set_title(&mut self, title: String) {
        self.title = title;
    }

    fn get_title(&self) -> &str {
        &self.title
    }
}

impl EventApi for Select {
    fn handle_input(&mut self, control: Controls) {
        match control {
            Controls::Right => {
                if self.options.len() > 1 && self.selected < self.options.len() - 1 {
                    self.selected += 1;
                }
            }
            Controls::Left => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            Controls::Enter => {
                self.channel.send(Message::Select(self.selected)).unwrap();
                std::thread::sleep(std::time::Duration::from_millis(42));
                self.channel.send(Message::Close).unwrap();
            }
            _ => {}
        };
    }
}
