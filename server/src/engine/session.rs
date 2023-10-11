use crate::engine::player::Message;

use super::event::Event;
use super::player::{EqPlayer, New, Player, Reciver, Splittable, TcpPlayer, TcpReciver, Whole};
use super::rules::{self, RuleEngine};
use async_trait::async_trait;
use std::cell::{RefCell, RefMut};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::time::sleep;

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

pub trait LobbyInterface {
    /// Connects a specific player to a specific session   
    fn connect<P: Player + 'static>(&mut self, player: Box<RefCell<P>>)
        -> Result<(), SessionError>;
    /// Disconnects a player from a session
    ///
    /// The player is stored in a temporary queue to allow reconnects
    fn disconnect(&mut self, player: usize) -> Result<(), SessionError>;
    /// Closes the session
    fn close(self) -> Vec<Box<RefCell<dyn Player>>>;
}

pub trait PlayerFromTcpStream<const BUFFERSIZE: usize, const CAPACITY: usize> {
    fn add<
        P: Player + Splittable<TcpReciver<BUFFERSIZE>> + 'static,
        T: New<P> + EqPlayer,
    >(
        &mut self,
        user: T,
    ) -> (usize, broadcast::Receiver<super::player::Message>);
}

pub struct Lobby<R: RuleEngine, const CAPACITY: usize> {
    players: Vec<Box<RefCell<dyn Player>>>,
    disconnected: Vec<(usize, Box<RefCell<dyn Player>>)>,
    rules: R,
    message_queue: Arc<Box<Mutex<Vec<(usize, Event)>>>>,

    user_counter: usize,
}

impl<R: RuleEngine, const CAPACITY: usize> LobbyInterface for Lobby<R, CAPACITY> {
    /// Closes the session
    fn close(self) -> Vec<Box<RefCell<dyn Player>>> {
        // Maybe we should notify the players here.
        self.players
    }

    /// Connects a specific player to a specific session   
    fn connect<P: Player + 'static>(
        &mut self,
        player: Box<RefCell<P>>,
    ) -> Result<(), SessionError> {
        println!("{:?}", self.players.len());

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
    fn disconnect(&mut self, player: usize) -> Result<(), SessionError> {
        let mut id = None;
        for (idx, el) in self.players.iter().enumerate() {
            if player == el.borrow().getid() {
                id = Some(idx);
                break;
            }
        }
        println!("Deleting user {:?}", id);
        // This should move the player to some intermediate place so a player can recover their connection
        match id {
            Some(idx) => {
                let ret = self.players.remove(idx);
                self.disconnected.push((player, ret));
                Ok(())
            }
            None => Err(SessionError::NoSuchPlayer),
        }
    }
}
impl<R: RuleEngine + rules::New, const CAPACITY: usize, const BUFFERSIZE: usize>
    PlayerFromTcpStream<BUFFERSIZE, CAPACITY> for Lobby<R, CAPACITY>
{
    fn add<
        P: Player + Splittable<TcpReciver<BUFFERSIZE>> + 'static,
        T: New<P> + EqPlayer,
    >(
        &mut self,
        user: T,
    ) -> (usize, broadcast::Receiver<super::player::Message>) {
        // Check if player is disconnected or not

        let mut uid = self.user_counter;
        self.user_counter += 1;

        for (idx, (old_uid, old_player)) in self.disconnected.iter().enumerate() {
            let old_player = old_player.as_mut().borrow();
            if user.eq(old_player) {
                uid = *old_uid;
                println!("Matched!");
                self.disconnected.remove(idx);
                break;
            }
        }

        let (player, mut receiver) = user.new(uid).split();

        self.players.push(Box::new(RefCell::new(player)));
        let subscriber: broadcast::Receiver<Message> = receiver.subscribe().unwrap();
        tokio::spawn(async move {
            let _ = receiver.recive().await;
        });

        (self.user_counter - 1, subscriber)
    }
}

impl<R: RuleEngine + rules::New, const CAPACITY: usize> Lobby<R, CAPACITY> {
    pub fn new<ID: Sized>(id: ID, mut channel: MessageBuss) -> Self {
        let mut msg_queue = Arc::new(Box::new(Mutex::new(Vec::with_capacity(CAPACITY))));

        let queue = RefCell::new(Arc::new(msg_queue.clone()));
        // Monitor for events
        tokio::spawn(async move {
            loop {
                let rec: Option<(usize, Event)> = channel.recv().await;
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
            disconnected: Vec::new(),
            rules: R::new(),
            message_queue: msg_queue,
            user_counter: 0,
        }
    }
    fn messages(&mut self) -> Vec<(usize, Event)> {
        let mut msg = self.message_queue.lock().unwrap();
        let mut ret = Vec::new();
        while let Some(message) = msg.pop() {
            ret.push(message);
        }
        ret
    }
    /// Starts the game.
    ///
    /// This function allows the usage of tokio to spawn game tasks.
    pub async fn start(lobby_ref: Arc<Mutex<RefCell<Self>>>) {
        println!("In lobby");
        loop {
            Self::run(lobby_ref.clone()).await;
            sleep(Duration::from_millis(100)).await;
        }
    }

    /// Manages the game logic
    async fn run(lobby_ref: Arc<Mutex<RefCell<Self>>>) {
        let lobby_mutex = lobby_ref.lock().unwrap();
        let mut lobby = lobby_mutex.try_borrow_mut().unwrap();

        let messages = lobby.messages();

        for message in messages {
            match lobby.rules.register_message(&vec![0, 1, 2, 3, 4], &message) {
                Ok(_) => {}
                Err(_) => {}
            }
            println!("cmd : {:?}", message);
        }
        drop(lobby);
        drop(lobby_mutex);

        //lobby.message_queue.lock().unwrap().as_mut().a;
    }
}
