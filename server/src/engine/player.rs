use crate::engine::event::Event;
use async_trait::async_trait;
use std::{io::Read, io::Write, net::TcpStream};

#[derive(Debug)]
pub enum PlayerError {
    /// Thrown when no response was delivered within the acceptable time
    TimeOut,
    /// Thrown when tcp write all breaks
    SendMessageError,
}

trait Connected {}
trait Disconnected {}

#[async_trait]
pub trait Player {
    fn getid(&self) -> usize;
    fn send(&mut self, event: Event) -> Result<(), PlayerError>;
    async fn recive(&mut self) -> Result<Event, PlayerError>;
}

// A player can fully be reprsented by a tcp stream, we just need to add functionality for it
#[derive(Debug)]
pub struct TcpPlayer {
    stream: TcpStream,
    id: usize,
}

#[async_trait]
impl Player for TcpPlayer {
    fn send(&mut self, event: Event) -> Result<(), PlayerError> {
        let data: Vec<u8> = (&event).into();
        match self.stream.write_all(&data[..]) {
            Ok(_) => Ok(()),
            Err(_) => {
                println!("Failed to send {:?} to {:?}", event, self);
                Err(PlayerError::SendMessageError)
            }
        }
    }
    async fn recive(&mut self) -> Result<Event, PlayerError> {
        let mut buffer = [0; 128];
        match self.stream.read(&mut buffer) {
            Err(_) => return Err(PlayerError::SendMessageError),
            _ => {}
        };
        match Vec::from(buffer).try_into() {
            Ok(event) => Ok(event),
            Err(_) => Err(PlayerError::SendMessageError),
        }
    }
    fn getid(&self) -> usize {
        return self.id.clone();
    }
}

impl TcpPlayer {
    pub fn new(stream: TcpStream, id: usize) -> Self {
        Self { stream, id }
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
