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
pub struct Info {
    channel: broadcast::Sender<Message>,
    title: String,
}

impl Info {
    pub fn new(channel: broadcast::Sender<Message>, title: String) -> Self {
        Self { channel, title }
    }
}
impl Popup for Info {
    fn subscribe(&mut self) -> broadcast::Receiver<Message> {
        self.channel.subscribe()
    }
}
impl TuiPage for Info {
    fn draw<B: ratatui::prelude::Backend>(
        &mut self,
        frame: &mut ratatui::Frame<B>,
        block: ratatui::prelude::Rect,
    ) {
        let block = Block::default().borders(Borders::ALL);
        let paragraph = Paragraph::new(self.title.blue())
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - 20) / 2),
                Constraint::Percentage(20),
                Constraint::Percentage((100 - 20) / 2),
            ])
            .split(frame.size());

        let area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - 30) / 2),
                Constraint::Percentage(30),
                Constraint::Percentage((100 - 30) / 2),
            ])
            .split(popup_layout[1])[1];
        let paragraph_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - 40) / 2),
                Constraint::Percentage(100),
            ])
            .split(area)[1];
        frame.render_widget(Clear, area); //this clears out the background
        frame.render_widget(block, area);
        frame.render_widget(paragraph, paragraph_area);

        info!("Popup drawn!");
    }

    fn set_title(&mut self, title: String) {
        self.title = title;
    }

    fn get_title(&mut self) -> &str {
        &self.title
    }
}
impl EventApi for Info {
    fn handle_input(&mut self, control: Controls) {
        match control {
            Controls::Enter => {
                self.channel.send(Message::Close).unwrap();
                info!("Closing the popup");
            }
            _ => {}
        };
    }
}
