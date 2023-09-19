mod tcp;
pub use tcp::*;

use crate::engine::event::Event;
use async_trait::async_trait;
use tokio;
use tokio::sync::broadcast::Receiver;

#[derive(Debug, Clone, Copy)]
pub enum PlayerError {
    /// Thrown when no response was delivered within the acceptable time
    TimeOut,

    /// Thrown when tcp write all breaks
    SendMessageError,

    /// Thrown when a player did not respond to KeepAlive
    Disconnected,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Recived {
        event: Result<Event, PlayerError>,
        user: usize,
    },
}

#[async_trait]
pub trait Reciver: std::fmt::Debug {
    fn subscribe(&mut self) -> Result<Receiver<Message>, PlayerError>;
    async fn recive(mut self) -> Result<(), PlayerError>;
}

#[async_trait]
pub trait Player: std::fmt::Debug + Send {
    fn getid(&self) -> usize;
    async fn send(&mut self, event: Event) -> Result<(), PlayerError>;
}
pub trait Splittable<R: Reciver> {
    type WritePart: Player;
    fn split(self) -> (Self::WritePart, R);
}

pub trait New<O: Player> {
    fn new(self, uid: usize) -> O;
}

#[cfg(test)]
mod test {
    use std::net::TcpListener;
    #[test]
    fn send_tcp() {
        // 1. start

        unimplemented!();
    }
}
