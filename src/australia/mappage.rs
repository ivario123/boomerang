use std::marker::PhantomData;

use ratatui::{
    prelude::{Backend, Constraint, Direction, Layout, Rect},
    symbols::Marker,
    widgets::canvas::Canvas,
    Frame,
};
use tokio::sync::Mutex;

use tui::{
    maps::{self, sites::Region, Map},
    tui::{controls::EventApi, TuiPage},
};

pub struct DefaultTuiMap<M: maps::Map> {
    map: M,
    title: String,
    visited: Vec<char>,
}

impl<M: maps::Map> DefaultTuiMap<M> {
    pub fn new() -> Self {
        Self {
            map: M::default(),
            title: "Map".to_owned(),
            visited: Vec::new(),
        }
    }
    pub fn update_visited(&mut self, visited: Vec<char>) {
        self.visited = visited
    }
}

impl<M: maps::Map> EventApi for DefaultTuiMap<M> {
    fn handle_input(&mut self, _control: tui::tui::controls::Controls) {
        // This should be able to assign scores and things I guess
    }
}

impl<M: maps::Map + ratatui::widgets::canvas::Shape> TuiPage for DefaultTuiMap<M>
where
    M::REGION: 'static,
{
    fn get_title(&self) -> &str {
        &self.title
    }
    fn set_title(&mut self, title: String) {
        self.title = title
    }
    fn draw<B: Backend>(&mut self, frame: &mut Frame<B>, block: Rect) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
            .split(block);

        let canvas = Canvas::default()
            .x_bounds([0.0, M::WIDTH as f64])
            .y_bounds([0.0, M::HEIGHT as f64])
            .paint(|context| {
                let mut sites = self.map.render(context);

                let mut region = <M as Map>::REGION::default();
                let mut offset = 0;
                let _ = sites.iter_mut().enumerate().for_each(|(idx, site)| {
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

        let scoring_region = frame.render_widget(canvas, layout[0]);
    }
}
