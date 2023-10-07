use ratatui::Frame;
use ratatui::backend::Backend;
use ratatui::prelude::*;

pub trait Drawable<B:Backend>{
    fn draw(&self,frame: &mut Frame<B>,area:Rect);
}
