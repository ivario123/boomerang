//! Defines a [`Score`] popup
//!
//! This popup is shown at the end of the game

use log::{error, info};
use ratatui::{
    prelude::{Constraint, Direction, Layout},
    style::Stylize,
    widgets::{Block, Borders, Clear},
};
use tokio::sync::broadcast;
use tui::tui::{
    controls::{Controls, EventApi},
    popup::{Message, Popup},
    TuiPage,
};

use crate::australia::{rules::scoring::Scoring, tui::ScoreList};

#[derive(Debug)]
pub struct Score {
    id: u8,
    scores: Vec<(u8, Scoring)>,
}

impl Score {
    pub fn new(id: u8, scores: Vec<(u8, Scoring)>) -> Self {
        Self { id, scores }
    }
}
impl Popup for Score {
    fn subscribe(&mut self) -> broadcast::Receiver<Message> {
        error!("");
        unreachable!()
    }
    fn exit(&mut self) {}
}
impl TuiPage for Score {
    fn draw<B: ratatui::prelude::Backend>(
        &mut self,
        frame: &mut ratatui::Frame<B>,
        _block: ratatui::prelude::Rect,
    ) {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - 90) / 2),
                Constraint::Percentage(90),
                Constraint::Percentage((100 - 90) / 2),
            ])
            .split(frame.size());

        let area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - 90) / 2),
                Constraint::Percentage(90),
                Constraint::Percentage((100 - 90) / 2),
            ])
            .split(popup_layout[1])[1];

        let backdrop = Block::default().borders(Borders::ALL).title("Final scores");
        frame.render_widget(Clear, area);
        frame.render_widget(backdrop, area);
        let scoring_cols = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area)
            .to_vec();
        let trophy = |idx:usize| -> String {
            match idx {
                0 => "🏆",
                1 => "🥈",
                2 => "🥉",
                _ => ""
            }.to_owned()
        };
        for (idx, col) in scoring_cols.iter().enumerate() {
            let score_area = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(*col);
            if let Some((id, score)) = self.scores.get(idx) {
                let label1 = format!("#{:?} : You {}", idx + 1,trophy(idx));
                let label2 = format!("#{:?} : Player {:?} {}", idx + 1, id,trophy(idx));
                let block = Block::default()
                    .borders(Borders::ALL)
                    .title(match *id == self.id {
                        true => label1.slow_blink(),
                        false => label2.gray(),
                    });
                ScoreList(vec![(*score).clone()]).draw(frame, block.inner(score_area[0]));
                frame.render_widget(block, score_area[0]);
            }
            if let Some((id, score)) = self.scores.get(idx + 2) {
                let label1 = format!("#{:?} : You {}", idx + 1,trophy(idx));
                let label2 = format!("#{:?} : Player {:?} {}", idx + 1, id,trophy(idx));
                let block = Block::default()
                    .borders(Borders::ALL)
                    .title(match *id == self.id {
                        true => label1.slow_blink(),
                        false => label2.gray(),
                    });
                ScoreList(vec![(*score).clone()]).draw(frame, block.inner(score_area[1]));
                frame.render_widget(block, score_area[1]);
            }
        }

        info!("Popup drawn!");
    }

    fn set_title(&mut self, _title: String) {}

    fn get_title(&self) -> &str {
        "Final Result"
    }
}
impl EventApi for Score {
    fn handle_input(&mut self, control: Controls) {
        match control {
            _ => {}
        };
    }
}
