use ratatui::{
    prelude::{Backend, Rect},
    Frame,
};

use crate::{tui::TuiPage, ui::Hand};

use super::card::TuiCard;

pub struct DefaultMainPage<H: Hand<TuiCard>> {
    hand: H,
    title: String,
}
impl<H: Hand<TuiCard>> DefaultMainPage<H> {
    pub fn new() -> Self {
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
