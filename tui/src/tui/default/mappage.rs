use ratatui::{
    prelude::{Backend, Rect},
    symbols::Marker,
    widgets::canvas::Canvas,
    Frame,
};

use crate::{
    maps::{self, sites::Region, Map},
    tui::{controls::EventApi, TuiPage},
};

pub struct DefaultTuiMap<M: maps::Map> {
    map: M,
    title: String,
}

impl<M: maps::Map> DefaultTuiMap<M> {
    pub fn new() -> Self {
        Self {
            map: M::default(),
            title: "Map".to_owned(),
        }
    }
}

impl<M: maps::Map> EventApi for DefaultTuiMap<M> {
    fn handle_input(&mut self, control: crate::tui::controls::Controls) {
        // This should be able to assign scores and things I guess
    }
}

impl<M: maps::Map + ratatui::widgets::canvas::Shape> TuiPage for DefaultTuiMap<M>
where
    M::REGION: 'static,
{
    fn get_title(&mut self) -> &str {
        &self.title
    }
    fn set_title(&mut self, title: String) {
        self.title = title
    }
    fn draw<B: Backend>(&mut self, frame: &mut Frame<B>, block: Rect) {
        let canvas = Canvas::default()
            .x_bounds([0.0, M::WIDTH as f64])
            .y_bounds([0.0, M::HEIGHT as f64])
            .paint(|context| {
                let mut sites = self.map.render(context);

                let mut region = <M as Map>::REGION::default();
                let mut offset = 0;
                let _ = sites.iter_mut().enumerate().for_each(|(idx, site)| {
                    if idx % 2 == 0 {
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

        frame.render_widget(canvas, block);
    }
}
