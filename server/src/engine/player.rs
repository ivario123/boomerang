mod tcp;
pub use tcp::*;
use async_trait::async_trait;
use tokio;
use tokio::sync::broadcast;

use super::event;

#[derive(Debug, Clone, Copy)]
pub enum PlayerError {
    /// Thrown when no response was delivered within the acceptable time
    _TimeOut,

    /// Thrown when tcp write all breaks
    SendMessageError,

    /// Thrown when a player did not respond to KeepAlive
    Disconnected,
}

#[derive(Debug, Clone)]
pub enum Message<Event: event::GameEvent> {
    Received {
        event: Result<Event, PlayerError>,
        user: usize,
    },
}

#[async_trait]
pub trait Receiver<Event: event::GameEvent>: std::fmt::Debug {
    fn subscribe(&mut self) -> Result<broadcast::Receiver<Message<Event>>, PlayerError>;
    async fn receive(mut self) -> Result<(), PlayerError>;
}
pub trait ReassignUid {
    fn re_assign_uid(self) -> Self;
}

#[async_trait]
pub trait Player<Event: event::GameEvent>: std::fmt::Debug + Send {
    fn get_id(&self) -> usize;
    async fn send(&mut self, event: Event) -> Result<(), PlayerError>;
    fn send_blocking(&mut self, event: Event) -> Result<(), PlayerError> {
        async_std::task::block_on(async {
            let async_result = self.send(event).await;
            async_result
        })
    }
    fn identifier(&self) -> String;
}

pub trait Id {
    fn identifier(&self) -> String;
}

pub trait EqPlayer {
    fn identifier(&self) -> String;
    fn eq(&self, other: impl Id) -> bool {
        self.identifier().to_owned() == other.identifier()
    }
}

impl<Event: event::GameEvent> dyn Player<Event> {
    #[allow(dead_code)]
    fn eq(&self, other: impl Id) -> bool {
        self.identifier() == other.identifier()
    }
}

pub trait Split<Event: event::GameEvent, const BUFFER_SIZE: usize> {
    type WritePart: Player<Event>;
    type ReadPart: Receiver<Event>;
    fn split(self) -> (Self::WritePart, Self::ReadPart);
}

pub trait New<Event: event::GameEvent, const CAPACITY: usize> {
    type Output: Player<Event>;
    fn new(self, uid: usize) -> Self::Output;
}
