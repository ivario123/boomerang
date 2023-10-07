use ratatui::{
    prelude::{Backend, Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders},
    Frame,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub trait UiElement {
    fn new() -> Self;
}

pub trait Card: UiElement {}

pub trait Hand<C: Card>: UiElement {
    fn get<const COUNT: usize>(&self, start: usize) -> &[C];
}

#[async_trait::async_trait]
pub trait Ui {
    async fn start(ui: Arc<Mutex<Box<Self>>>);
}
