pub mod info;
pub mod select;
use tokio::sync::broadcast::{self, Receiver};

use super::{
    controls::{Controls, EventApi},
    TuiPage,
};

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Close,
    Select(usize),
}

pub trait Popup: TuiPage + std::fmt::Debug {
    fn subscribe(&mut self) -> Receiver<Message>;
}
