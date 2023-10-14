use crate::engine::player::Message;

use super::event::Event;
use super::player::{
    self, EqPlayer, Id, New, Player, PlayerError, Reciver, Splittable, TcpReciver,
};
use super::rules::{self, Action, RuleEngine};
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::time::sleep;

#[derive(Debug)]
pub enum SessionError {
    /// Thrown when a disconnect is requested for a non exsisting player
    NoSuchPlayer,
    /// Thrown when a player is trying to connect to a full lobby
    LobbyFull,
    /// Thrown when a that player is already connected to the game
    PlayerAlreadyConnected,
    /// [`Player::send`] threw some error
    PlayerError(PlayerError),
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
    fn add<P: Player + Splittable<TcpReciver<BUFFERSIZE>> + Id + 'static, T: New<P> + EqPlayer>(
        &mut self,
        user: T,
    ) -> Result<(usize, broadcast::Receiver<super::player::Message>), SessionError>;
}

pub struct Lobby<R: RuleEngine, const CAPACITY: usize> {
    players: Vec<Box<RefCell<dyn Player>>>,
    disconnected: Vec<(usize, Box<RefCell<dyn Player>>)>,
    rules: R,
    message_queue: Arc<Box<Mutex<Vec<rules::Action<rules::New>>>>>,
    event_queue: Vec<rules::Action<rules::Sent>>,
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
impl<R: RuleEngine + rules::Instantiable, const CAPACITY: usize, const BUFFERSIZE: usize>
    PlayerFromTcpStream<BUFFERSIZE, CAPACITY> for Lobby<R, CAPACITY>
{
    fn add<P: Player + Splittable<TcpReciver<BUFFERSIZE>> + Id + 'static, T: New<P> + EqPlayer>(
        &mut self,
        user: T,
    ) -> Result<(usize, broadcast::Receiver<super::player::Message>), SessionError> {
        // Check if player is disconnected or not

        for connected_player in &self.players {
            if user.identifier() == connected_player.borrow().identifier() {
                return Err(SessionError::PlayerAlreadyConnected);
            }
        }

        let mut uid = self.user_counter;
        self.user_counter += 1;
        let mut remove_idx = None;
        for (idx, (old_uid, old_player)) in self.disconnected.iter_mut().enumerate() {
            let old_player = old_player.as_mut().borrow();
            println!(
                "Comparing {:?} == {:?}",
                user.identifier(),
                old_player.identifier()
            );
            if user.identifier() == old_player.identifier() {
                uid = *old_uid;
                println!("Matched!");
                remove_idx = Some(idx);
                break;
            }
        }
        if let Some(idx) = remove_idx {
            self.disconnected.remove(idx);
        }

        let (player, mut receiver) = user.new(uid).split();

        self.players.push(Box::new(RefCell::new(player)));
        let subscriber: broadcast::Receiver<Message> = receiver.subscribe().unwrap();
        tokio::spawn(async move {
            let _ = receiver.receive().await;
        });

        Ok((self.user_counter - 1, subscriber))
    }
}

impl<R: RuleEngine + rules::Instantiable, const CAPACITY: usize> Lobby<R, CAPACITY> {
    pub fn new<ID: Sized>(_: ID, mut channel: MessageBuss) -> Self {
        let msg_queue = Arc::new(Box::new(Mutex::new(Vec::with_capacity(CAPACITY))));

        let queue = RefCell::new(Arc::new(msg_queue.clone()));
        // Monitor for events
        tokio::spawn(async move {
            loop {
                let rec: Option<(usize, Event)> = channel.recv().await;
                match rec {
                    Some((player, event)) => {
                        queue
                            .borrow_mut()
                            .lock()
                            .unwrap()
                            .push(Action::<rules::New>::new(player, event));
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
            event_queue: Vec::new(),
            message_queue: msg_queue,
            user_counter: 0,
        }
    }

    /// Flushes the messages from the message queue returning the flushed messages
    ///
    /// Returns a [`Vec`] of events and the corresponding [`Player`] [`Id`](Player::getid).
    fn flush_messages(&mut self) -> Vec<Action<rules::New>> {
        let mut msg = self.message_queue.lock().unwrap();
        let mut ret = Vec::new();
        while let Some(message) = msg.pop() {
            println!("flushing {:?}", message);
            ret.push(message);
        }
        ret
    }

    async fn send_message(
        lobby_ref: &Arc<Mutex<RefCell<Self>>>,
        action: Action<rules::New>,
    ) -> Result<Action<rules::Sent>, Action<rules::New>> {
        let lobby_ref_lock = lobby_ref.lock();
        let lobby_locked = match lobby_ref_lock {
            Ok(lobby) => lobby,
            _ => return Err(action),
        };
        let mut lobby = match lobby_locked.try_borrow_mut() {
            Ok(lobby) => lobby,
            Err(_) => return Err(action),
        };
        let mut disconnect = None;
        let mut found_player = None;
        for player_ref in lobby.players.iter_mut() {
            let uid = player_ref.get_mut().getid();
            if uid == action.player() {
                found_player = Some(player_ref.get_mut());
            }
        }
        if let Some(player) = found_player {
            let id = player.getid();
            let action = action.action();
            disconnect = match player.send_blocking(action) {
                Err(PlayerError::Disconnected) => Some(id),
                _ => None,
            }
        };
        if let Some(player_idx) = disconnect {
            // We know that the player exists in the list we just saw it.
            // And we are still holding the lock on the lobby
            lobby.disconnect(player_idx).unwrap();
        };
        Ok(action.transition())
    }

    /// Starts the game.
    ///
    /// This function allows the usage of tokio to spawn game tasks.
    pub async fn start(lobby_ref: Arc<Mutex<RefCell<Self>>>) {
        println!("In lobby");
        loop {
            Self::main(lobby_ref.clone()).await;
            sleep(Duration::from_millis(100)).await;
        }
    }

    fn enqueue(&mut self, action: Result<Action<rules::Sent>, Action<rules::New>>) {
        match action {
            Ok(action) => {
                self.event_queue.push(action);
            }
            Err(action) => {
                let mut message_queue = self.message_queue.borrow_mut();
                let queue = match (&mut message_queue).lock() {
                    Ok(message_queue) => Some(message_queue),
                    Err(_) => None,
                };
                if let Some(mut queue) = queue {
                    queue.push(action);
                }
            }
        }
    }

    /// Manages the game logic
    async fn main(lobby_ref: Arc<Mutex<RefCell<Self>>>) {
        // We should add a broadcast channel to the game lobby that shuts it down if this panics
        // for now it is better to just panic the thread if an error occurs here
        let lobby_mutex = lobby_ref.lock().unwrap();

        let mut lobby = match lobby_mutex.try_borrow_mut() {
            Ok(lobby) => lobby,
            // The error means that we can't acquire a mutable borrow
            // at this time, if so, return and retry in a bit
            Err(_) => return,
        };

        let messages = lobby.flush_messages();
        let mut send_queue = Vec::new();

        for action in messages {
            let rules = &mut lobby.rules;
            let uid = action.player();
            println!("cmd : {:?}", action);
            match rules.register_message(&vec![0, 1, 2, 3, 4], &action) {
                Ok(_) => {}
                Err(rules::Error::UnexpectedResponse((event, action))) => {
                    send_queue.push(action);
                }
                Err(rules::Error::UnexpectedMessage) => {
                    send_queue.push(Action::<rules::New>::new(uid, Event::UnexpectedMessage));
                }
                Err(e) => {
                    println!("{:?}", e);
                }
            }
        }
        for action in send_queue {
            println!("{:?}", action);
            let ret = async_std::task::block_on(Self::send_message(&lobby_ref, action));
            println!("{:?}", ret);
            lobby.enqueue(ret);
        }

        //lobby.message_queue.lock().unwrap().as_mut().a;
    }
}
