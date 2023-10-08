use ratatui::{prelude::Backend, Frame};

use crate::{
    tui::TuiPage,
    ui::{self, Card, Hand, UiElement},
};

use super::mainpage::DefaultMainPage;

pub type TuiCard = u8;

pub struct TuiHand {
    cards: Vec<TuiCard>,
}

impl UiElement for TuiCard {
    fn new() -> Self {
        todo!()
    }
}

impl Card for TuiCard {}


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
