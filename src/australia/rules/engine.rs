//! Defines the rule engine for the boomerang australia game
use server::engine::rules::{RuleEngine, New, Action, Received, Error, Completed, Instantiable};

use crate::australia::protocol::Event;

use super::states::{GameState, WaitingForPlayers, DealingCards};

pub struct Australia<const CAPACITY: usize, const MIN_PLAYERS: usize> {
    state: Box<dyn GameState>,
}

impl<const CAPACITY: usize, const MIN_PLAYERS: usize> RuleEngine
    for Australia<CAPACITY, MIN_PLAYERS>
{
    type Event = Event;
    fn get_next_action(
        &mut self,
        players: &Vec<usize>,
    ) -> (tokio::time::Duration, Vec<Action<New, Self::Event>>) {
        let (delay, actions, state) = self.state.get_next_action(players);
        if let Some(state) = state {
            self.state = state;
        }

        (delay, actions)
    }

    fn register_response(
        &mut self,
        _: &Vec<usize>,
        response: (Self::Event, &Action<Received, Self::Event>),
    ) -> Result<Action<Completed, Self::Event>, Error> {
        let completed_action =
            Action::<Completed, Self::Event>::new(response.1.player(), response.1.action().clone());
        let res = self.state.register_response(response);
        match res {
            Ok(val) => {
                match val {
                    Some(state) => self.state = state,
                    None => {}
                }
                Ok(completed_action)
            }
            Err(e) => Err(e),
        }
    }

    fn register_message(
        &mut self,
        _players: &Vec<usize>,
        _message: &Action<New, Self::Event>,
    ) -> Result<(), Error> {
        return Err(Error::UnexpectedMessage);
    }
}
impl<const CAPACITY: usize, const MIN_PLAYERS: usize> Instantiable
    for Australia<CAPACITY, MIN_PLAYERS>
{
    fn new() -> Self {
        Australia {
            state: Box::new(WaitingForPlayers::<DealingCards>::new(None)),
        }
    }
}