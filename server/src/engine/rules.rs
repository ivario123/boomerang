use std::cell::RefCell;

use super::{event::Event, player::Player, session::Session};

pub struct Action {
    player: usize,
    action: Event,
}

pub enum Error {
    /// Thrown when the response recieved is not the expected one.
    UnexpectedResponse((Event, Action)),
}

pub trait RuleEngine {
    /// Returns the next set of actions, this could be 1 action or it could be many.
    /// Also it returns the minimum time to wait before requesting any new actions,
    /// this is use full if the game is in a wait state, say that not all players have
    /// selected a throw card.
    fn get_next_action(&mut self, players: &Vec<usize>) -> (tokio::time::Instant, Vec<Action>);
    fn register_response(
        &mut self,
        players: &Vec<usize>,
        respone: (Event, Action),
    ) -> Result<(), Error>;
    fn register_message(
        &mut self,
        players: &Vec<usize>,
        message: &(usize, Event),
    ) -> Result<(), Error>;
}
pub trait New {
    fn new() -> Self;
}

pub struct Austrailia<const CAPACITY: usize> {}

impl<const CAPACITY: usize> RuleEngine for Austrailia<CAPACITY> {
    fn get_next_action(&mut self, players: &Vec<usize>) -> (tokio::time::Instant, Vec<Action>) {
        (tokio::time::Instant::now(), Vec::new())
    }

    fn register_response(
        &mut self,
        players: &Vec<usize>,
        respone: (Event, Action),
    ) -> Result<(), Error> {
        Ok(())
    }

    fn register_message(
        &mut self,
        players: &Vec<usize>,
        message: &(usize, Event),
    ) -> Result<(), Error> {
        Ok(())
    }
}
impl<const CAPACITY: usize> New for Austrailia<CAPACITY> {
    fn new() -> Self {
        Austrailia {}
    }
}
