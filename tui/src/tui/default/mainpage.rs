use std::fmt::format;

use ratatui::{
    layout,
    prelude::{Backend, Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders},
    Frame,
};

use crate::{
    tui::{
        controls::{Controls, EventApi},
        TuiPage,
    },
    ui::{Card, Hand},
};

use super::card::{self, TuiCard};

pub struct DefaultMainPage<H: Hand<TuiCard>> {
    hand: H,
    title: String,
    card_disp_ptr: usize,
}
impl<H: Hand<TuiCard>> DefaultMainPage<H> {
    const COUNT: usize = 3;
    pub fn new() -> Self {
        Self {
            hand: H::new(),
            title: "Game".to_owned(),
            card_disp_ptr: 0,
        }
    }
}

impl<H: Hand<TuiCard>> EventApi for DefaultMainPage<H> {
    fn handle_input(&mut self, control: Controls) {
        match control {
            Controls::Left => match self.card_disp_ptr > 0 {
                true => self.card_disp_ptr -= 1,
                _ => {}
            },
            Controls::Right => match self.card_disp_ptr < self.hand.count() - 1 {
                true => self.card_disp_ptr += 1,
                _ => {}
            },
            _ => {}
        }
    }
}

impl<H: Hand<TuiCard>> TuiPage for DefaultMainPage<H> {
    fn draw<B: Backend>(&mut self, frame: &mut Frame<B>, block: Rect) {
        let (cards, (last, count)) = self.hand.get::<3>(self.card_disp_ptr);

        let card_area = Block::default()
            .title(format!(
                "Hand (showing {:?} - {:?}/{:?})",
                self.card_disp_ptr + 1,
                last,
                count
            ))
            .borders(Borders::all());
        frame.render_widget(card_area, block);

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Percentage((100 / Self::COUNT) as u16); 3].as_ref())
            .split(block);

        for (area, card) in layout.iter().zip(cards) {
            let rect = Block::default()
                .title(card.get_name())
                .borders(Borders::all());
            frame.render_widget(rect, *area);
        }
    }
    fn set_title(&mut self, title: String) {
        self.title = title
    }
    fn get_title(&mut self) -> &str {
        &self.title
    }
}
