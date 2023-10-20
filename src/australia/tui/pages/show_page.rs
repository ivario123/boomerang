//! Defines the page where other players show hand is shown
use std::marker::PhantomData;

use ratatui::{
    prelude::{Backend, Constraint, Direction, Layout, Rect},
    style::Color,
    symbols::Marker,
    widgets::canvas::Canvas,
    Frame,
};

use tui::{
    maps::{sites::Region, Map as MapTrait},
    tui::{
        controls::{Controls, EventApi},
        show_page::ShowPage as ShowPageTrait,
        TuiPage,
    },
    ui::{Card, Hand},
};


use crate::australia::{tui::map::australia::Map, rules::cards::AustraliaCard};

use super::main_page::CardArea;

pub struct ShowPage<C: Card, H: Hand<C> + CardArea<C>> {
    discard_pile: H,
    card: PhantomData<C>,
    title: String,
    map: Map,
    visited: Vec<char>,
}
impl<C: Card, H: Hand<C> + CardArea<C>> ShowPage<C, H> {
    pub fn new(uid: usize, showing: H, visited: Vec<char>) -> Self {
        Self {
            discard_pile: showing,
            map: Map::default(),
            card: PhantomData,
            visited,
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

        let canvas = Canvas::default()
            .x_bounds([0.0, Map::WIDTH as f64])
            .y_bounds([0.0, Map::HEIGHT as f64])
            .paint(|context| {
                let mut sites = self.map.render(context);

                let mut region = <Map as MapTrait>::REGION::default();
                let mut offset = 0;
                let _ = sites.iter_mut().enumerate().for_each(|(_idx, site)| {
                    if self.visited.contains(&site.get_id()) {
                        site.complete();
                    }
                    let new_region = site.region();
                    if region != new_region {
                        region = new_region;
                        offset = 0;
                    }
                    site.clone().render(context, (offset * 10) as f64);
                    offset += 1;
                });
            })
            .marker(Marker::Dot);
        frame.render_widget(canvas, layout[2])
    }
    fn set_title(&mut self, title: String) {
        self.title = title
    }
    fn get_title(&self) -> &str {
        &self.title
    }
}
