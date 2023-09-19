use super::event::{self, Event};
use super::player::{New, Player, Reciver, Splittable, TcpReciver};
use super::rules::{self, RuleSet};
use super::{monitor, player};
use async_trait::async_trait;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tokio::sync::mpsc;

pub enum SessionError {
    /// Thrown when a disconnect is requested for a non exsisting player
    NoSuchPlayer,
    /// Thrown when a player is trying to connect to a full lobby
    LobbyFull,
}

pub type MessageBuss = mpsc::Receiver<(usize, Event)>;

pub trait Session {
    type Error;
    fn new() -> Self;
    fn delete(&mut self, uid: usize) -> Result<Box<RefCell<dyn Player>>, Self::Error>;
    fn add<R: Reciver, P: Player + Splittable<R> + 'static, T: New<P>>(
        &mut self,
        user: T,
    ) -> (usize, dyn Reciver);
}

#[async_trait]
pub trait LobbyInterface {
    /// Connects a specific player to a specific session   
    fn connect<P: Player + 'static>(&mut self, player: Box<RefCell<P>>)
        -> Result<(), SessionError>;
    /// Disconnects a player from a session
    fn disconnect(&mut self, player: usize) -> Result<Box<RefCell<dyn Player>>, SessionError>;
    /// Closes the session
    fn close(self) -> Vec<Box<RefCell<dyn Player>>>;
}

pub trait PlayerFromTcpStream<const BUFFERSIZE: usize> {
    fn add<P: Player + Splittable<TcpReciver<BUFFERSIZE>> + 'static, T: New<P>>(
        &mut self,
        user: T,
    ) -> (usize, broadcast::Receiver<super::player::Message>);
}

pub struct Lobby<R: RuleSet, const CAPACITY: usize> {
    players: Vec<Box<RefCell<dyn Player>>>,
    rules: R,
    message_queue: Arc<Box<Mutex<Vec<(usize, Event)>>>>,
    user_counter: usize,
}

#[async_trait]
impl<R: RuleSet, const CAPACITY: usize> LobbyInterface for Lobby<R, CAPACITY> {
    /// Closes the session
    fn close(self) -> Vec<Box<RefCell<dyn Player>>> {
        // Maybe we should notify the players here.
        self.players
    }
    // Some trait things
    /// Connects a specific player to a specific session   
    fn connect<P: Player + 'static>(
        &mut self,
        player: Box<RefCell<P>>,
    ) -> Result<(), SessionError> {
        let num_players = self.players.len();
        match num_players >= CAPACITY {
            true => Err(SessionError::LobbyFull),
            _ => {
                self.players.push(player);
                Ok(())
            }
        }
    }
    /// Disconnects a player from a session
    fn disconnect(&mut self, player: usize) -> Result<Box<RefCell<dyn Player>>, SessionError> {
        let mut id = None;
        for (idx, el) in self.players.iter().enumerate() {
            if player == el.borrow().getid() {
                id = Some(idx);
                break;
            }
        }
        println!("Deleting user {:?}", id);
        match id {
            Some(idx) => Ok(self.players.remove(idx)),
            None => Err(SessionError::NoSuchPlayer),
        }
    }
}
impl<R: RuleSet + rules::New, const CAPACITY: usize, const BUFFERSIZE: usize>
    PlayerFromTcpStream<BUFFERSIZE> for Lobby<R, CAPACITY>
{
    fn add<P: Player + Splittable<TcpReciver<BUFFERSIZE>> + 'static, T: New<P>>(
        &mut self,
        user: T,
    ) -> (usize, broadcast::Receiver<super::player::Message>) {
        let (player, mut reciver) = user.new(self.user_counter).split();
        self.players.push(Box::new(RefCell::new(player)));
        self.user_counter += 1;
        let subscriber = reciver.subscribe().unwrap();
        tokio::spawn(async move {
            reciver.recive().await;
        });
        (self.user_counter - 1, subscriber)
    }
}

impl<R: RuleSet + rules::New, const CAPACITY: usize> Lobby<R, CAPACITY> {
    pub fn new<ID: Sized>(id: ID, mut channel: MessageBuss) -> Self {
        let mut msg_queue = Arc::new(Box::new(Mutex::new(Vec::with_capacity(CAPACITY))));

        let queue = RefCell::new(Arc::new(msg_queue.clone()));
        tokio::spawn(async move {
            loop {
                let rec = channel.recv().await;
                match rec {
                    Some(msg) => {
                        println!("Recived {:?}", msg);
                        queue.borrow_mut().lock().unwrap().push(msg);
                    }
                    _ => {
                        eprintln!("Nothing to see here");
                    }
                };
            }
        });

        Self {
            players: Vec::with_capacity(CAPACITY),
            rules: R::new(),
            message_queue: msg_queue,
            user_counter: 0,
        }
    }
    pub async fn start(mut self) {
        self.message_queue.lock().unwrap().pop();
        loop {}
    }

    // Called when session goes out of scope.
    fn free(self) {
        self.close();
    }
}
