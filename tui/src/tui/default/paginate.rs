use ratatui::{
    prelude::{Backend, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Tabs},
    Frame,
};

use crate::tui::TuiPage;

pub struct Paginate<MainPage: TuiPage, MapPage: TuiPage>(MainPage, MapPage, usize);
impl<MainPage: TuiPage, MapPage: TuiPage> Paginate<MainPage, MapPage> {
    pub fn increment(&mut self) {
        self.2 = match self.2 {
            0 => 1,
            1 => 0,
            _ => unreachable!(),
        }
    }
    pub fn decrement(&mut self) {
        self.2 = match self.2 {
            0 => 1,
            1 => 0,
            _ => unreachable!(),
        }
    }
    pub fn new(mainpage: MainPage, mappage: MapPage) -> Self {
        Self(mainpage, mappage, 0)
    }
    pub fn draw<B: Backend>(&mut self, frame: &mut Frame<B>, pagination_area: Rect, block: Rect) {
        // We need to draw either
        let titles = [self.0.get_title(), self.1.get_title()].to_vec();
        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title("Tabs"))
            .select(self.2)
            .style(Style::default().cyan())
            .highlight_style(Style::default().bold().on_black());

        frame.render_widget(tabs, pagination_area);

        match self.2 {
            0 => self.0.draw(frame, block),
            1 => self.1.draw(frame, block),
            _ => unreachable!(),
        };
    }
}
