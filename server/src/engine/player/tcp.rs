use super::{EqPlayer, Id, Message, New, Player, PlayerError, Reciver};
use crate::engine::event::{Event, EventList};
use async_std::task;
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use tokio::net::{
    tcp::{OwnedReadHalf, OwnedWriteHalf},
    TcpStream,
};
use tokio::sync::broadcast::{self, Receiver, Sender};

pub trait TcpPlayerState: std::fmt::Debug + Send {}
#[derive(Debug)]
pub struct Whole {}
#[derive(Debug)]
pub struct WriteEnabled {}

impl TcpPlayerState for Whole {}
impl TcpPlayerState for WriteEnabled {}

// A player can fully be reprsented by a tcp stream, we just need to add functionality for it
#[derive(Debug)]
pub struct TcpPlayer<const CAPACITY: usize, STATE: TcpPlayerState> {
    pub writer: Mutex<OwnedWriteHalf>,
    reader: Option<Mutex<OwnedReadHalf>>,
    id: usize,
    sender: Option<Mutex<Sender<Message>>>,
    state: std::marker::PhantomData<STATE>,
}

#[async_trait]
impl<const CAPACITY: usize, STATE: TcpPlayerState> Player for TcpPlayer<CAPACITY, STATE> {
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
    fn identifier(&self) -> String {
        format!(
            "TcpPlayer, Peer : {:?}",
            self.writer.lock().unwrap().peer_addr()
        )
    }
}
impl Id for std::net::TcpStream {
    fn identifier(&self) -> String {
        format!("TcpPlayer, Peer : {:?}", self.peer_addr())
    }
}
impl EqPlayer for std::net::TcpStream {
    fn identifier(&self) -> String {
        format!("TcpPlayer, Peer : {:?}", self.peer_addr())
    }
}

impl<const CAPACITY: usize, const BUFFERSIZE: usize> super::Splittable<TcpReciver<BUFFERSIZE>>
    for TcpPlayer<CAPACITY, Whole>
{
    type WritePart = TcpPlayer<CAPACITY, WriteEnabled>;
    fn split(self) -> (Self::WritePart, TcpReciver<BUFFERSIZE>) {
        let Some(reader) = self.reader else {
            unreachable!()
        };
        let id = self.id.clone();
        let Some(sender) = self.sender else {
            unreachable!()
        };
        (
            TcpPlayer {
                reader: None,
                writer: self.writer,
                sender: None,
                id: self.id,
                state: std::marker::PhantomData,
            },
            TcpReciver {
                reader,
                id,
                sender: Arc::new(Box::new(sender)),
            },
        )
    }
}

impl<const CAPACITY: usize, STATE: TcpPlayerState> TcpPlayer<CAPACITY, STATE> {
    pub fn new(stream: TcpStream, id: usize) -> Self {
        let (sender, _rx) = broadcast::channel(CAPACITY);
        let (reader, writer) = stream.into_split();
        let (reader, writer) = (Mutex::new(reader), Mutex::new(writer));
        let ret = Self {
            reader: Some(reader),
            writer,
            id,
            sender: Some(Mutex::new(sender)),
            state: std::marker::PhantomData,
        };
        ret
    }
}

impl<const CAPACITY: usize> New<TcpPlayer<CAPACITY, Whole>> for std::net::TcpStream {
    fn new(self, uid: usize) -> TcpPlayer<CAPACITY, Whole> {
        let stream = TcpStream::from_std(self).unwrap();
        TcpPlayer::new(stream, uid)
    }
}

#[derive(Debug)]
pub struct TcpReciver<const CAPACITY: usize> {
    reader: Mutex<OwnedReadHalf>,
    id: usize,
    sender: Arc<Box<Mutex<Sender<Message>>>>,
}
#[async_trait]
impl<const CAPACITY: usize> Reciver for TcpReciver<CAPACITY> {
    fn subscribe(&mut self) -> Result<Receiver<Message>, PlayerError> {
        let sender = self.sender.lock().unwrap();
        Ok(sender.subscribe())
    }

    async fn receive(mut self) -> Result<(), PlayerError> {
        loop {
            let mut buffer = [0; 128];
            let readable = self.reader.lock().unwrap();
            match readable.try_read(&mut buffer) {
                Err(_) => return Err(PlayerError::SendMessageError),
                Ok(n) => {
                    if n == 0 {
                        self.sender
                            .lock()
                            .unwrap()
                            .send(Message::Recived {
                                event: Err(PlayerError::Disconnected),
                                user: self.id.clone(),
                            })
                            .unwrap();
                        return Ok(());
                    }
                }
            };
            drop(readable);
            let EventList(events) = Vec::from(buffer).into();
            for event in events.iter() {
                let msg = match event {
                    Ok(event) => Ok(event.clone()),
                    Err(_) => Err(PlayerError::SendMessageError),
                };
                let sender_clone: Arc<Box<Mutex<Sender<Message>>>> = self.sender.clone();

                tokio::spawn({
                    async move {
                        let msg = Message::Recived {
                            event: msg,
                            user: self.id.clone(),
                        };
                        let send = |sender: Arc<Box<Mutex<Sender<Message>>>>| {
                            let sender_locked = match sender.lock() {
                                Ok(valid_sender) => valid_sender,
                                // There is no way to recover from this.
                                Err(_) => return,
                            };

                            match sender_locked.send(msg) {
                                Ok(_) => {}
                                Err(_) => {
                                    // Here we need to recurse with some counter to terminate the
                                    // loop if some threshold is exceeded.
                                }
                            };
                        };
                        send(sender_clone.clone());
                    }
                });
            }
        }
    }
}
