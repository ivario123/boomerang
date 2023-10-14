use std::{cell::RefCell, marker::PhantomData};

use super::{
    event::{self, BackendEvent, GameEvent},
    player::Player,
    session::Session,
};

pub trait ActionStatus {}

macro_rules! impl_status {
    (New) => {
        impl<Event:GameEvent> Action<New,Event>{
            fn from_action<Status:ActionStatus>(action:Action<Status,Event>) -> Self{
                Self{
                    player:action.player,
                    action:action.action,
                    status:PhantomData
                }
            }
        }
    };
    ($status:ident) => {
        impl<Event:GameEvent> Into<Action<New,Event>> for Action<$status,Event>{
            fn into(self) -> Action<New,Event> {
                Action::<New,Event>::from_action(self)
            }
        }
    };
    (all $($status:ident)+) => {
        $(
            impl ActionStatus for $status {}
            impl_status!($status);
        )+
    };
}
macro_rules! transition_of_status {
    ($($status1:ident -> $status2:ident)+) => {
        $(
            impl<Event:GameEvent>  Action<$status1,Event> {
                pub fn transition(self) -> Action<$status2,Event>{
                    Action::<$status2,Event>{
                        player:self.player,
                        action:self.action,
                        status:PhantomData
                    }
                }
            }
        )+
    };
}
#[derive(Debug)]
pub struct New {}
#[derive(Debug)]
pub struct Sent {}
#[derive(Debug)]
pub struct Received {}
#[derive(Debug)]
pub struct Completed {}
impl_status!(all New Sent Received Completed);
transition_of_status!(New -> Sent Sent -> Received Received -> Completed);

#[derive(Debug)]
pub struct Action<Status: ActionStatus, Event: GameEvent> {
    player: usize,
    action: Event,
    status: PhantomData<Status>,
}

impl<Status: ActionStatus, Event: GameEvent> Action<Status, Event> {
    pub fn player(&self) -> usize {
        self.player
    }
    pub fn action(&self) -> Event {
        self.action.clone()
    }
    pub fn new(player: usize, action: Event) -> Action<New, Event> {
        Action {
            player,
            action,
            status: PhantomData,
        }
    }
}
#[derive(Debug)]
pub enum Error {
    /// Thrown when the response recieved is not the expected one.
    ///
    /// Wraps the requested action to resolve this and the event that triggered the error
    UnexpectedResponse,
    /// Thrown when a un prompted un expected event occurs.
    UnexpectedMessage,
}

pub trait RuleEngine {
    type Event: event::GameEvent + Send;

    /// Returns the next set of actions, this could be 1 action or it could be many.
    /// Also it returns the minimum time to wait before requesting any new actions,
    /// this is use full if the game is in a wait state, say that not all players have
    /// selected a throw card.
    fn get_next_action(
        &mut self,
        players: &Vec<usize>,
    ) -> (tokio::time::Instant, Vec<Vec<Action<New, Self::Event>>>);
    fn register_response(
        &mut self,
        players: &Vec<usize>,
        response: (Self::Event, Action<Received, Self::Event>),
    ) -> Result<Action<Completed, Self::Event>, Error>;
    fn register_message(
        &mut self,
        players: &Vec<usize>,
        message: &Action<New, Self::Event>,
    ) -> Result<(), Error>;
}

pub trait Instantiable {
    fn new() -> Self;
}

pub struct Austrailia<const CAPACITY: usize> {}

impl<const CAPACITY: usize> RuleEngine for Austrailia<CAPACITY> {
    type Event = BackendEvent;
    fn get_next_action(
        &mut self,
        players: &Vec<usize>,
    ) -> (tokio::time::Instant, Vec<Vec<Action<New, Self::Event>>>) {
        (tokio::time::Instant::now(), Vec::new())
    }

    fn register_response(
        &mut self,
        players: &Vec<usize>,
        response: (Self::Event, Action<Received, Self::Event>),
    ) -> Result<Action<Completed, Self::Event>, Error> {
        Err(Error::UnexpectedResponse)
    }

    fn register_message(
        &mut self,
        players: &Vec<usize>,
        message: &Action<New, Self::Event>,
    ) -> Result<(), Error> {
        return Err(Error::UnexpectedMessage);
    }
}
impl<const CAPACITY: usize> Instantiable for Austrailia<CAPACITY> {
    fn new() -> Self {
        Austrailia {}
    }
}
