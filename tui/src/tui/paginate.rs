use crossterm::cursor::Show;
use log::info;
use ratatui::{
    prelude::{Backend, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Tabs},
    Frame,
};

use crate::tui::{controls::EventApi, showpage::ShowPage as ShowPageTrait, TuiPage};

pub struct Paginate<MainPage: TuiPage, MapPage: TuiPage, ShowPage: ShowPageTrait>(
    MainPage,
    MapPage,
    Vec<ShowPage>,
    usize,
);

impl<MainPage: TuiPage, MapPage: TuiPage, ShowPage: ShowPageTrait> EventApi
    for Paginate<MainPage, MapPage, ShowPage>
{
    fn handle_input(&mut self, control: crate::tui::controls::Controls) {
        match control {
            crate::tui::controls::Controls::Tab => self.increment(),
            crate::tui::controls::Controls::Exit => {}
            _ => {
                match self.3 {
                    0 => self.0.handle_input(control),
                    1 => self.1.handle_input(control),
                    mut x => {
                        x -= 2;
                        info!("Trying to draw show page {:?}", x);
                        if x < self.2.len() {
                            self.2[x].handle_input(control);
                            return;
                        }
                    }
                };
            }
        }
    }
}

impl<MainPage: TuiPage, MapPage: TuiPage, ShowPage: ShowPageTrait>
    Paginate<MainPage, MapPage, ShowPage>
{
    pub fn increment(&mut self) {
        self.3 = match self.3 {
            0 => 1,
            mut x => {
                x += 1;
                if x > self.2.len() + 1 {
                    x = 0;
                }
                x
            }
        }
    }
    pub fn decrement(&mut self) {
        self.3 = match self.3 {
            0 => self.2.len() + 1,
            x => x - 1,
        }
    }
    pub fn new(mainpage: MainPage, mappage: MapPage) -> Self {
        Self(mainpage, mappage, Vec::new(), 0)
    }
    pub fn add_show_page(&mut self, page: ShowPage) {
        self.2.push(page);
    }
    pub fn replace_into(&mut self, page: ShowPage) {
        let mut found = None;
        for (idx, el) in self.2.iter().enumerate() {
            if el.eq(&page) {
                found = Some(idx);
                break;
            }
        }
        match found {
            Some(idx) => self.2[idx] = page,
            None => self.2.push(page),
        }
    }
    pub fn draw<B: Backend>(&mut self, frame: &mut Frame<B>, pagination_area: Rect, block: Rect) {
        // We need to draw either
        let mut titles: Vec<&str> = vec![self.0.get_title(), self.1.get_title()];
        for page in &self.2 {
            titles.push(page.get_title())
        }

        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title("Tabs"))
            .select(self.3)
            .style(Style::default().cyan())
            .highlight_style(Style::default().bold().on_black());

        frame.render_widget(tabs, pagination_area);

        match self.3 {
            0 => self.0.draw(frame, block),
            1 => self.1.draw(frame, block),
            mut x => {
                x -= 2;
                info!("Trying to draw show page {:?}", x);
                if x < self.2.len() {
                    self.2[x].draw(frame, block);
                    return;
                }
            }
        };
    }

    pub fn main_page(&mut self) -> &mut MainPage {
        &mut self.0
    }
}
