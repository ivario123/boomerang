use std::sync::Arc;
use tokio::sync::RwLock;

pub trait UiMessage: Clone + std::fmt::Debug {}

pub trait UiElement {
    fn new() -> Self;
}

pub trait Card {
    fn get_name(&self) -> &str;
    fn number(&self) -> usize;
}

pub trait Hand<C: Card>: UiElement {
    fn get<const COUNT: usize>(&self, start: usize) -> (&[C], (usize, usize));
    fn add_card(&mut self, card: C);
    fn discard_card(&mut self, idx: usize) -> C;
    fn count(&self) -> usize;
}

#[async_trait::async_trait]
pub trait Ui {
    async fn start(ui: Arc<RwLock<Box<Self>>>);
}
