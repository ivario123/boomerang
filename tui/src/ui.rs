use std::sync::Arc;
use tokio::sync::Mutex;

pub trait UiElement {
    fn new() -> Self;
}

pub trait Card: UiElement {
    fn get_name(&self) -> &str;
}

pub trait Hand<C: Card>: UiElement {
    fn get<const COUNT: usize>(&self, start: usize) -> (&[C], (usize, usize));
    fn count(&self) -> usize;
}

#[async_trait::async_trait]
pub trait Ui {
    async fn start(ui: Arc<Mutex<Box<Self>>>);
}
