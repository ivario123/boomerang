mod tcp;
pub use tcp::*;

use crate::engine::event::GameEvent;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::any::TypeId;
use std::cell::RefCell;
use std::cmp::PartialOrd;
use tokio;
use tokio::sync::broadcast;

use super::event::{self, EventError};
#[derive(Debug, Clone, Copy)]
pub enum PlayerError {
    /// Thrown when no response was delivered within the acceptable time
    TimeOut,

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
pub trait ReasignUid {
    fn re_asign_uid(self) -> Self;
}

#[async_trait]
pub trait Player<Event: event::GameEvent>: std::fmt::Debug + Send {
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


pub trait EqPlayer {
    fn identifier(&self) -> String;
    fn eq(&self, other: impl Id) -> bool {
        self.identifier().to_owned() == other.identifier()
    }
}

impl<Event: event::GameEvent> dyn Player<Event> {
    fn eq(&self, other: impl Id) -> bool {
        self.identifier() == other.identifier()
    }
}

pub trait Splittable<Event: event::GameEvent,const BUFFERSIZE:usize> {
    type WritePart: Player<Event>;
    type ReadPart: Receiver<Event>;
    fn split(self) -> (Self::WritePart, Self::ReadPart);
}

pub trait New<Event:event::GameEvent,const CAPACITY:usize> {
    type Output:Player<Event>;
    fn new(self, uid: usize) -> Self::Output;
}
