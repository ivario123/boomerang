use super::{EqPlayer, Id, Message, New, Player, PlayerError};
use crate::engine::event::GameEvent;
use async_trait::async_trait;
use std::marker::PhantomData;
use tokio::net::{
    tcp::{OwnedReadHalf, OwnedWriteHalf},
    TcpStream,
};
use tokio::sync::broadcast::{self, Receiver, Sender};
use tokio::sync::Mutex;
pub trait TcpPlayerState: std::fmt::Debug + Send {}
#[derive(Debug)]
pub struct Whole {}
#[derive(Debug)]
pub struct WriteEnabled {}

impl TcpPlayerState for Whole {}
impl TcpPlayerState for WriteEnabled {}

// A player can fully be reprsented by a tcp stream, we just need to add functionality for it
#[derive(Debug)]
pub struct TcpPlayer<const CAPACITY: usize, STATE: TcpPlayerState, Event: GameEvent> {
    pub mutex: Mutex<PhantomData<STATE>>,
    pub writer: OwnedWriteHalf,
    reader: Option<OwnedReadHalf>,
    id: usize,
    sender: Option<Sender<Message<Event>>>,
    state: std::marker::PhantomData<STATE>,
}

#[derive(Debug)]
pub struct TcpReciver<const CAPACITY: usize, Event: GameEvent> {
    reader: OwnedReadHalf,
    id: usize,
    sender: Sender<Message<Event>>,
    mutex: Mutex<PhantomData<bool>>,
}

#[async_trait]
impl<const CAPACITY: usize, STATE: TcpPlayerState, Event: GameEvent> Player<Event>
    for TcpPlayer<CAPACITY, STATE, Event>
{
    async fn send(&mut self, event: Event) -> Result<(), PlayerError> {
        let _ = self.mutex.lock().await;
        println!("{:?} sending {:?}", self, event);
        let json = match serde_json::to_string(&event) {
            Ok(val) => val,
            Err(_) => return Err(PlayerError::SendMessageError),
        };
        match self.writer.try_write(json.as_bytes()) {
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
        format!("TcpPlayer, Peer : {:?}", self.writer.peer_addr())
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

impl<Event: GameEvent, const CAPACITY: usize, const BUFFERSIZE: usize>
    super::Splittable<Event, BUFFERSIZE> for TcpPlayer<CAPACITY, Whole, Event>
{
    type WritePart = TcpPlayer<CAPACITY, WriteEnabled, Event>;
    type ReadPart = TcpReciver<BUFFERSIZE, Event>;
    fn split(self) -> (Self::WritePart, TcpReciver<BUFFERSIZE, Event>) {
        let Some(reader) = self.reader else {
            unreachable!()
        };
        let id = self.id.clone();
        let Some(sender) = self.sender else {
            unreachable!()
        };
        (
            TcpPlayer {
                mutex: Mutex::new(PhantomData),
                reader: None,
                writer: self.writer,
                sender: None,
                id: self.id,
                state: std::marker::PhantomData,
            },
            TcpReciver {
                reader,
                id,
                sender: sender,
                mutex: Mutex::new(PhantomData),
            },
        )
    }
}

impl<const CAPACITY: usize, STATE: TcpPlayerState, Event: GameEvent>
    TcpPlayer<CAPACITY, STATE, Event>
{
    pub fn new(stream: TcpStream, id: usize) -> Self {
        let (sender, _rx) = broadcast::channel(CAPACITY);
        let (reader, writer) = stream.into_split();
        let (reader, writer) = (reader, writer);
        let ret = Self {
            mutex: Mutex::new(PhantomData),
            reader: Some(reader),
            writer,
            id,
            sender: Some(sender),
            state: std::marker::PhantomData,
        };
        ret
    }
}

impl<const CAPACITY: usize, Event: GameEvent> New<Event, CAPACITY> for std::net::TcpStream {
    type Output = TcpPlayer<CAPACITY, Whole, Event>;
    fn new(self, uid: usize) -> Self::Output {
        let stream = TcpStream::from_std(self).unwrap();
        TcpPlayer::new(stream, uid)
    }
}
impl<const CAPACITY: usize, Event: GameEvent, State: TcpPlayerState> Id
    for TcpPlayer<CAPACITY, State, Event>
{
    fn identifier(&self) -> String {
        format!("TcpPlayer, Peer : {:?}", self.writer.peer_addr())
    }
}

#[async_trait]
impl<const CAPACITY: usize, Event: GameEvent + Sync> crate::engine::player::Receiver<Event>
    for TcpReciver<CAPACITY, Event>
{
    fn subscribe(&mut self) -> Result<Receiver<Message<Event>>, PlayerError> {
        let sender = &self.sender;
        Ok(sender.subscribe())
    }

    async fn receive(mut self) -> Result<(), PlayerError> {
        loop {
            let mut buffer = vec![0; 128];
            {
                let _ = self.mutex.lock().await;
                match self.reader.try_read(&mut buffer) {
                    Err(_) => {}
                    Ok(n) => {
                        if n == 0 {
                            self.sender
                                .send(Message::Received {
                                    event: Err(PlayerError::Disconnected),
                                    user: self.id.clone(),
                                })
                                .unwrap();
                            return Ok(());
                        }
                    }
                };
            }
            let events: Vec<Event> = {
                let _ = self.mutex.lock().await;
                while let Some(&0) = buffer.last() {
                    buffer.pop();
                }
                match serde_json::from_slice::<Vec<Event>>(&buffer) {
                    Ok(vec) => vec,
                    // If it is not a vec of events, see if it is a single event
                    Err(_) => match serde_json::from_slice::<Event>(&buffer) {
                        Ok(event) => vec![event],
                        Err(_) => {
                            continue;
                        }
                    },
                }
            };

            for event in events.iter() {
                {
                    let _ = self.mutex.lock().await;
                    // Re package in to a nice little message
                    let msg: Message<Event> = Message::Received {
                        event: Ok(event.clone()),
                        user: self.id.clone(),
                    };
                    self.sender.send(msg).unwrap();
                }
                println!("Sent to manager");
            }
        }
    }
}
