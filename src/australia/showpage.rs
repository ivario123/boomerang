use std::{borrow::BorrowMut, marker::PhantomData};

use async_std::channel;
use log::info;
use ratatui::{
    prelude::{Backend, Constraint, Direction, Layout, Rect},
    style::Color,
    Frame,
};
use tokio::sync::{broadcast, Mutex};

use tui::{
    tui::{
        controls::{Controls, EventApi},
        showpage::ShowPage as ShowPageTrait,
        TuiPage,
    },
    ui::{Card, Hand},
};

use crate::rules::cards::AustraliaCard;

use super::{mainpage::CardArea, Message};

use std::panic;

pub struct ShowPage<C: Card, H: Hand<C> + CardArea<C>> {
    discard_pile: H,
    card: PhantomData<C>,
    title: String,
}
impl<C: Card, H: Hand<C> + CardArea<C>> ShowPage<C, H> {
    const COUNT: usize = 4;
    pub fn new(uid: usize, showing: H) -> Self {
        Self {
            discard_pile: showing,
            card: PhantomData,
            title: format!("Player {}", uid).to_owned(),
        }
    }
}

impl<H: Hand<AustraliaCard> + CardArea<AustraliaCard>> ShowPageTrait
    for ShowPage<AustraliaCard, H>
{
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title
    }
}

impl<H: Hand<AustraliaCard> + CardArea<AustraliaCard>> EventApi for ShowPage<AustraliaCard, H> {
    fn handle_input(&mut self, control: Controls) {
        match control {
            Controls::Left => self.discard_pile.decrement(),
            Controls::Right => self.discard_pile.increment(),
            _ => {}
        }
    }
}

impl<H: Hand<AustraliaCard> + CardArea<AustraliaCard>> TuiPage for ShowPage<AustraliaCard, H> {
    fn draw<B: Backend>(&mut self, frame: &mut Frame<B>, block: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(45),
                    Constraint::Percentage(5),
                    Constraint::Percentage(45),
                ]
                .as_ref(),
            )
            .split(block);
        self.discard_pile
            .draw(frame, layout[0], "Hand", Color::White);
    }
    fn set_title(&mut self, title: String) {
        self.title = title
    }
    fn get_title(&self) -> &str {
        &self.title
    }
}
