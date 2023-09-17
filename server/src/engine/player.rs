use crate::engine::event::Event;
use async_trait::async_trait;
use std::sync::Mutex;
use std::{future::Future, io::Read, io::Write, net::TcpStream, pin::Pin};
use tokio;
use tokio::sync::broadcast::{Receiver, Sender, self};

#[derive(Debug,Clone,Copy)]
pub enum PlayerError {
    /// Thrown when no response was delivered within the acceptable time
    TimeOut,

    /// Thrown when tcp write all breaks
    SendMessageError,

    /// Thrown when a player did not respond to KeepAlive
    Disconnected,
}

#[derive(Debug,Clone, Copy)]
pub enum Message {
    Recived {
        event: Result<Event,PlayerError>,
        user: usize,
    },
}

trait Connected {}
trait Disconnected {}

#[async_trait]
pub trait Player: std::fmt::Debug {
    fn getid(&self) -> usize;
    async fn send(&mut self, event: Event) -> Result<(), PlayerError>;
    fn subscribe(&mut self) -> Result<Receiver<Message>, PlayerError>;
    async fn recive(&mut self) -> Result<(), PlayerError>;
}

// A player can fully be reprsented by a tcp stream, we just need to add functionality for it
#[derive(Debug)]
pub struct TcpPlayer<const CAPACITY:usize> {
    stream: Mutex<TcpStream>,
    id: usize,
    sender: Sender<Message>,
}

#[async_trait]
impl<const CAPACITY:usize> Player for TcpPlayer<CAPACITY> {
    async fn send(&mut self, event: Event) -> Result<(), PlayerError> {
        let data: Vec<u8> = (&event).into();
        let writeable = self.stream.lock().unwrap();
        match writeable.write_all(&data[..]) {
            Ok(_) => Ok(()),
            Err(_) => {
                println!("Failed to send {:?} to {:?}", event, self);
                Err(PlayerError::SendMessageError)
            }
        }
    }

    fn subscribe(&mut self) -> Result<Receiver<Message>, PlayerError> {
        Ok(self.sender.subscribe())
    }

    async fn recive(&mut self) -> Result<(), PlayerError> {
        loop {
            let mut buffer = [0; 128];
            let readable = self.stream.lock().unwrap();
            match readable.read(&mut buffer) {
                Err(_) => return Err(PlayerError::SendMessageError),
                _ => {}
            };
            drop(readable);

            match Vec::from(buffer).try_into() {
                Ok(event) => {
                    match event {
                        Event::KeepAlive => {
                            tokio::spawn(self.send(Event::KeepAliveResponse));
                        }
                        _ => {}
                    };
                    self.sender.send(Message::Recived {
                        event: Ok(event),
                        user: self.id,
                    });
                }
                Err(_) => {
                    self.sender.send(Message::Recived {
                        event: Err(PlayerError::SendMessageError),
                        user: self.id,
                    });
                }
            };
        }
    }
    fn getid(&self) -> usize {
        return self.id.clone();
    }
}

impl<const CAPACITY:usize> TcpPlayer<CAPACITY> {

    pub fn new(stream: TcpStream, id: usize) -> Self {
        let (sender,_rx) = broadcast::channel(CAPACITY);
        let mut ret = Self {
            stream: Mutex::new(stream),
            id,
            sender,
        };
        tokio::spawn(ret.recive());
        ret
    }
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
