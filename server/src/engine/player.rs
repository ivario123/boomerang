use crate::engine::event::Event;
use async_trait::async_trait;
use std::sync::Mutex;
use std::{future::Future, io::Read, io::Write, pin::Pin};
use tokio;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{tcp::ReadHalf, tcp::WriteHalf, TcpStream};
use tokio::sync::broadcast::{self, Receiver, Sender};

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

trait Connected {}
trait Disconnected {}
#[async_trait]
pub trait Reciver: std::fmt::Debug + Send{
    fn subscribe(&mut self) -> Result<Receiver<Message>, PlayerError>;
    async fn recive(mut self) -> Result<(), PlayerError>;

}

#[async_trait]
pub trait Player: std::fmt::Debug +Send{
    fn getid(&self) -> usize;
    async fn send(&mut self, event: Event) -> Result<(), PlayerError>;
}
pub trait Splittable{
    fn split<T:Reciver,P:Player+Sized>(self) -> (P,T);
}

pub trait TcpPlayerState:std::fmt::Debug+Send{}
#[derive(Debug)]
pub struct Whole{}
#[derive(Debug)]
pub struct WriteEnabled{}

impl TcpPlayerState for Whole{}
impl TcpPlayerState for WriteEnabled{}

// A player can fully be reprsented by a tcp stream, we just need to add functionality for it
#[derive(Debug)]
pub struct TcpPlayer<const CAPACITY: usize,STATE:TcpPlayerState> {
    writer: Mutex<OwnedWriteHalf>,
    reader: Option<Mutex<OwnedReadHalf>>,
    id: usize,
    sender: Option<Mutex<Sender<Message>>>,
    state:std::marker::PhantomData<STATE>
}

#[async_trait]
impl<const CAPACITY: usize,STATE:TcpPlayerState> Player for TcpPlayer<CAPACITY,STATE> {
    async fn send(&mut self, event: Event) -> Result<(), PlayerError> {
        let data: Vec<u8> = (&event).into();
        let writeable = self.writer.lock().unwrap();
        match writeable.try_write(&data[..]) {
            Ok(n) => {
                println!("Wrote {:?} bytes", n);
                Ok(())
            }
            Err(_) => {
                println!("Failed to send {:?} to {:?}", event, self);
                Err(PlayerError::SendMessageError)
            }
        }
    }


    fn getid(&self) -> usize {
        return self.id.clone();
    }
}

impl<const CAPACITY: usize> TcpPlayer<CAPACITY,Whole> {
   pub fn split(mut self) -> (TcpReciver<CAPACITY>,TcpPlayer<CAPACITY,WriteEnabled>){
        
        let Some(reader) = self.reader else {unreachable!()};
        let id = self.id.clone();
        let Some(sender) = self.sender else {unreachable!()};
        (TcpReciver{
            reader,id,sender
        },TcpPlayer{
            reader:None,
            writer:self.writer,
            sender:None,
            id:self.id,
            state:std::marker::PhantomData
        })
        
    } 
}
#[derive(Debug)]
pub struct TcpReciver<const CAPACITY:usize>{
    reader: Mutex<OwnedReadHalf>,
    id: usize,
    sender: Mutex<Sender<Message>>,

}

#[async_trait]
impl<const CAPACITY: usize> Reciver for TcpReciver<CAPACITY>{
    fn subscribe(&mut self) -> Result<Receiver<Message>, PlayerError> {
        let sender = self.sender.lock().unwrap();
        Ok(sender.subscribe())
    }

    async fn recive(mut self) -> Result<(), PlayerError> {
        loop {
            let mut buffer = [0; 128];
            let readable = self.reader.lock().unwrap();
            match readable.try_read(&mut buffer) {
                Err(_) => return Err(PlayerError::SendMessageError),
                Ok(n) => {
                    println!("Read {:?} bytes", n);
                    if n == 0{
                        self.sender.lock().unwrap().send(
                            Message::Recived{
                                event:Err(PlayerError::Disconnected),
                                user:self.id.clone()
                            }
                        ).unwrap();
                        return Ok(());
                    }
                }
            };
            drop(readable);

            let msg = match Vec::from(buffer).try_into() {
                Ok(event) => Ok(event),
                Err(_) => Err(PlayerError::SendMessageError),
            };

            let sender = self.sender.lock().unwrap();
            sender
                .send(Message::Recived {
                    event: msg,
                    user: self.id.clone(),
                })
                .unwrap();
            drop(sender);
        }
    }
}


impl<const CAPACITY: usize, STATE:TcpPlayerState> TcpPlayer<CAPACITY,STATE> {
    pub fn new(stream: TcpStream, id: usize) -> Self {
        let (sender, _rx) = broadcast::channel(CAPACITY);
        let (reader, writer) = stream.into_split();
        let (reader, writer) = (Mutex::new(reader), Mutex::new(writer));
        let mut ret = Self {
            reader:Some(reader),
            writer,
            id,
            sender: Some(Mutex::new(sender)),
            state: std::marker::PhantomData
        };
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
