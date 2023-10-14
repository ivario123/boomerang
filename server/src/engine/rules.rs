use std::{cell::RefCell, marker::PhantomData};

use super::{event::Event, player::Player, session::Session};

pub trait ActionStatus {}

macro_rules! impl_status {
    (New) => {
        impl Action<New>{
            fn from_action<Status:ActionStatus>(action:Action<Status>) -> Self{
                Self{
                    player:action.player,
                    action:action.action,
                    status:PhantomData
                }
            }
        }
    };
    ($status:ident) => {
        impl Into<Action<New>> for Action<$status>{
            fn into(self) -> Action<New> {
                Action::<New>::from_action(self)
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
            impl  Action<$status1> {
                pub fn transition(self) -> Action<$status2>{
                    Action::<$status2>{
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
pub struct Action<Status: ActionStatus> {
    player: usize,
    action: Event,
    status: PhantomData<Status>,
}

impl<Status: ActionStatus> Action<Status> {
    pub fn player(&self) -> usize {
        self.player
    }
    pub fn action(&self) -> Event {
        self.action
    }
    pub fn new(player: usize, action: Event) -> Action<New> {
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
    UnexpectedResponse((Event, Action<New>)),
    /// Thrown when a un prompted un expected event occurs.
    UnexpectedMessage,
}

pub trait RuleEngine {
    /// Returns the next set of actions, this could be 1 action or it could be many.
    /// Also it returns the minimum time to wait before requesting any new actions,
    /// this is use full if the game is in a wait state, say that not all players have
    /// selected a throw card.
    fn get_next_action(
        &mut self,
        players: &Vec<usize>,
    ) -> (tokio::time::Instant, Vec<Vec<Action<New>>>);
    fn register_response(
        &mut self,
        players: &Vec<usize>,
        response: (Event, Action<Received>),
    ) -> Result<Action<Completed>, Error>;
    fn register_message(
        &mut self,
        players: &Vec<usize>,
        message: &Action<New>,
    ) -> Result<(), Error>;
}

pub trait Instantiable {
    fn new() -> Self;
}

pub struct Austrailia<const CAPACITY: usize> {}

impl<const CAPACITY: usize> RuleEngine for Austrailia<CAPACITY> {
    fn get_next_action(
        &mut self,
        players: &Vec<usize>,
    ) -> (tokio::time::Instant, Vec<Vec<Action<New>>>) {
        (tokio::time::Instant::now(), Vec::new())
    }

    fn register_response(
        &mut self,
        players: &Vec<usize>,
        response: (Event, Action<Received>),
    ) -> Result<Action<Completed>, Error> {
        Err(Error::UnexpectedResponse((response.0, response.1.into())))
    }

    fn register_message(
        &mut self,
        players: &Vec<usize>,
        message: &Action<New>,
    ) -> Result<(), Error> {
        return Err(Error::UnexpectedMessage);
    }
}
impl<const CAPACITY: usize> Instantiable for Austrailia<CAPACITY> {
    fn new() -> Self {
        Austrailia {}
    }
}
