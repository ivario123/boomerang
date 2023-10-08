use ratatui::{
    prelude::{Backend, Rect},
    Frame,
};

pub mod default;

pub trait TuiPage {
    fn draw<B: Backend>(&mut self, frame: &mut Frame<B>, block: Rect);
    fn set_title(&mut self, title: String);
    fn get_title(&mut self) -> &str;
}
