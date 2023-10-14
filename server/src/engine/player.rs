mod tcp;
pub use tcp::*;

use crate::engine::event::Event;
use async_trait::async_trait;
use std::any::TypeId;
use std::cell::RefCell;
use std::cmp::PartialOrd;
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
    async fn receive(mut self) -> Result<(), PlayerError>;
}
pub trait ReasignUid {
    fn re_asign_uid(self) -> Self;
}

#[async_trait]
pub trait Player: std::fmt::Debug + Send {
    fn getid(&self) -> usize;
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

impl<P: Player> Id for P {
    fn identifier(&self) -> String {
        (self as &dyn Player).identifier()
    }
}

pub trait EqPlayer {
    fn identifier(&self) -> String;
    fn eq(&self, other: impl Id) -> bool {
        self.identifier().to_owned() == other.identifier()
    }
}

impl dyn Player {
    fn eq(&self, other: impl Id) -> bool {
        self.identifier() == other.identifier()
    }
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
