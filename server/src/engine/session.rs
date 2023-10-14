use crate::engine::event::BackendEvent;
use crate::engine::player::Message;

use super::event::GameEvent;
use super::player::{
    self, EqPlayer, Id, New, Player, PlayerError, Receiver, Splittable, TcpPlayer, TcpReciver,
    WriteEnabled,
};
use super::rules::{self, Action, RuleEngine};
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
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

pub type MessageBuss<Event: GameEvent> = mpsc::Receiver<(usize, Event)>;

pub trait Session<Event: GameEvent, const BUFFERSIZE: usize, const CAPACITY: usize> {
    type Error;
    fn new() -> Self;
    fn delete(&mut self, uid: usize) -> Result<Box<RefCell<dyn Player<Event>>>, Self::Error>;
    fn add<
        R: Receiver<Event>,
        P: Player<Event> + Splittable<Event, BUFFERSIZE, ReadPart = R> + 'static,
        T: New<Event, CAPACITY, Output = P>,
    >(
        &mut self,
        user: T,
    ) -> (usize, dyn Receiver<Event>);
}

pub trait LobbyInterface<Event: GameEvent> {
    /// Connects a specific player to a specific session   
    fn connect<P: Player<Event> + 'static>(
        &mut self,
        player: Box<RefCell<P>>,
    ) -> Result<(), SessionError>;
    /// Disconnects a player from a session
    ///
    /// The player is stored in a temporary queue to allow reconnects
    fn disconnect(&mut self, player: usize) -> Result<(), SessionError>;
    /// Closes the session
    fn close(self) -> Vec<Box<RefCell<dyn Player<Event>>>>;
}

pub trait PlayerFromTcpStream<const BUFFERSIZE: usize, const CAPACITY: usize, Event: GameEvent> {
    fn add<
        P: Player<Event>
            + Splittable<
                Event,
                BUFFERSIZE,
                WritePart = TcpPlayer<CAPACITY, WriteEnabled, Event>,
                ReadPart = TcpReciver<BUFFERSIZE, Event>,
            > + Id
            + 'static,
        T: New<Event, CAPACITY, Output = P> + EqPlayer,
    >(
        &mut self,
        user: T,
    ) -> Result<(usize, broadcast::Receiver<super::player::Message<Event>>), SessionError>;
}

pub struct Lobby<R: RuleEngine, const CAPACITY: usize> {
    players: Vec<Box<RefCell<dyn Player<R::Event>>>>,
    disconnected: Vec<(usize, Box<RefCell<dyn Player<R::Event>>>)>,
    rules: R,
    message_queue: Arc<Mutex<Vec<Action<rules::New, R::Event>>>>,
    event_queue: Vec<rules::Action<rules::Sent, R::Event>>,
    user_counter: usize,
}

impl<R: RuleEngine, const CAPACITY: usize> LobbyInterface<R::Event> for Lobby<R, CAPACITY> {
    /// Closes the session
    fn close(self) -> Vec<Box<RefCell<dyn Player<R::Event>>>> {
        // Maybe we should notify the players here.
        self.players
    }

    /// Connects a specific player to a specific session   
    fn connect<P: Player<R::Event> + 'static>(
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

impl<
        R: RuleEngine + rules::Instantiable + 'static,
        const CAPACITY: usize,
        const BUFFERSIZE: usize,
    > PlayerFromTcpStream<BUFFERSIZE, CAPACITY, R::Event> for Lobby<R, CAPACITY>
{
    fn add<
        P: Player<R::Event>
            + Splittable<
                R::Event,
                BUFFERSIZE,
                WritePart = TcpPlayer<CAPACITY, WriteEnabled, R::Event>,
                ReadPart = TcpReciver<BUFFERSIZE, R::Event>,
            > + Id
            + 'static,
        T: New<R::Event, CAPACITY, Output = P> + EqPlayer,
    >(
        &mut self,
        user: T,
    ) -> Result<(usize, broadcast::Receiver<super::player::Message<R::Event>>), SessionError> {
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
        let subscriber = receiver.subscribe().unwrap();
        tokio::spawn(async move {
            let _ = receiver.receive().await;
        });

        Ok((self.user_counter - 1, subscriber))
    }
}

impl<R: RuleEngine + rules::Instantiable + 'static, const CAPACITY: usize> Lobby<R, CAPACITY> {
    async fn monitor(
        channel: &mut MessageBuss<R::Event>,
        mut queue: Arc<Mutex<Vec<Action<rules::New, R::Event>>>>,
    ) {
        let (player, event) = match channel.try_recv() {
            Ok(value) => value,
            Err(_) => return,
        };

        let queue = queue.borrow_mut();
        let mut queue_locked = queue.lock().await;
        queue_locked.push(Action::<rules::New, R::Event>::new(player, event));
    }
    pub fn new<ID: Sized>(_: ID, mut channel: MessageBuss<R::Event>) -> Self {
        let msg_queue: Arc<Mutex<Vec<Action<rules::New, R::Event>>>> =
            Arc::new(Mutex::new(Vec::with_capacity(CAPACITY)));
        let queue: Arc<Mutex<Vec<Action<rules::New, R::Event>>>> = msg_queue.clone();
        // Monitor for events
        tokio::spawn(async move {
            loop {
                Self::monitor(&mut channel, queue.clone()).await
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
    fn flush_messages(&mut self) -> Vec<Action<rules::New, R::Event>> {
        let mut msg = async_std::task::block_on(async { self.message_queue.lock().await });
        let mut ret = Vec::new();
        while let Some(message) = msg.pop() {
            println!("flushing {:?}", message);
            ret.push(message);
        }
        ret
    }

    fn send_message(
        &mut self,
        action: Action<rules::New, R::Event>,
    ) -> Result<Action<rules::Sent, R::Event>, Action<rules::New, R::Event>> {
        let mut disconnect = None;
        let mut found_player = None;
        for player_ref in self.players.iter_mut() {
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
            self.disconnect(player_idx).unwrap();
        };
        Ok(action.transition())
    }

    fn enqueue(
        message_queue: &mut Vec<Action<rules::New, R::Event>>,
        event_queue: &mut Vec<rules::Action<rules::Sent, R::Event>>,
        action: Result<Action<rules::Sent, R::Event>, Action<rules::New, R::Event>>,
    ) {
        match action {
            Ok(action) => {
                event_queue.push(action);
            }
            Err(action) => {
                message_queue.push(action);
            }
        }
    }

    /// Manages the game logic
    fn main(&mut self) {
        // We should add a broadcast channel to the game lobby that shuts it down if this panics
        // for now it is better to just panic the thread if an error occurs here
        let messages = self.flush_messages();
        let mut send_queue = Vec::new();

        for action in messages {
            let rules = &mut self.rules;
            let uid = action.player();
            println!("cmd : {:?}", action);
            match rules.register_message(&vec![0, 1, 2, 3, 4], &action) {
                Ok(_) => {}
                Err(rules::Error::UnexpectedResponse) => {
                    send_queue.push(action);
                }
                Err(rules::Error::UnexpectedMessage) => {
                    send_queue.push(Action::<rules::New, R::Event>::new(
                        uid,
                        BackendEvent::UnexpectedMessage.into(),
                    ));
                }
                Err(e) => {
                    println!("{:?}", e);
                }
            }
        }
        for action in send_queue {
            println!("sending {:?}", action);
            let ret = self.send_message(action);
            println!("send returned {:?}", ret);
            let msg_queue: &mut Vec<Action<rules::New, R::Event>> =
                &mut async_std::task::block_on(async { self.message_queue.lock().await });
            let event_queue = &mut self.event_queue;
            Self::enqueue(msg_queue, event_queue, ret);
        }

        //lobby.message_queue.lock().unwrap().as_mut().a;
    }
}

// Split all of the async logic from the sync logic for readability

impl<R: RuleEngine + rules::Instantiable + 'static, const CAPACITY: usize> Lobby<R, CAPACITY> {
    /// Starts the game.
    ///
    /// This function allows the usage of tokio to spawn game tasks.
    pub async fn start(lobby_ref: Arc<Mutex<RefCell<Self>>>) {
        println!("In lobby");
        loop {
            Self::_start(lobby_ref.clone()).await;
            sleep(Duration::from_millis(100)).await;
        }
    }
    async fn _start(lobby_ref: Arc<Mutex<RefCell<Self>>>) {
        let lobby_lock = lobby_ref.lock().await;
        let mut lobby = match lobby_lock.try_borrow_mut() {
            Ok(lobby) => lobby,
            Err(_) => {
                return;
            }
        };
        lobby.main();
    }
}
