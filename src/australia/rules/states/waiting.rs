//! Defines a waiting state, this is the initial state although it could be used at later stages too.

use log::info;

use crate::{
    engine::rules::{Action, Error, New, Received}, australia::{protocol::Event, rules::meta::GameMetaData},
};

use super::{DealingCards, GameState, AsMetaData, WaitingForPlayers};

impl<Next: AsMetaData + Send + 'static> WaitingForPlayers<Next> {
    pub fn new(next_state: Option<Box<Next>>) -> Self {
        Self {
            ready: Vec::new(),
            pending_ready: Vec::new(),
            next_state: next_state,
        }
    }
}

impl<Next: AsMetaData + Send + 'static> GameState for WaitingForPlayers<Next> {
    fn get_next_action(
        &mut self,
        players: &Vec<usize>,
    ) -> (
        tokio::time::Duration,
        Vec<Action<New, Event>>,
        Option<Box<dyn GameState>>,
    ) {
        info!("State : {:?}", self);
        println!("In waiting state");

        let mut actions = Vec::new();

        if players.len() < 2 {
            for player in players {
                actions.push(Action::<New, Event>::new(*player, Event::WaitingForPlayers));
            }
        } else if players.len() > 4 {
            for player in players {
                actions.push(Action::<New, Event>::new(*player, Event::LobbyFull));
            }
        } else {
            // We have enough players, this means that all players need to be ready.
            if self.ready.len() == players.len() {
                // Now we know that the players are ready to go, let's start the game!
                // Go to the next state, this is likely the dealing cards state but if some
                // players disconnected it might be another state
                let state = std::mem::replace(&mut self.next_state, None);
                return (
                    tokio::time::Duration::from_millis(1000),
                    actions,
                    Some(match state {
                        Some(state) => state,
                        None => Box::new(DealingCards::new(players)),
                    }),
                );
            }
            for player in players {
                if !self.ready.contains(&(*player as u8))
                    && !self.pending_ready.contains(&(*player as u8))
                {
                    // This player is not ready
                    actions.push(Action::<New, Event>::new(*player, Event::ReadyCheck));
                    self.pending_ready.push(*player as u8);
                }
            }
        }
        return (tokio::time::Duration::from_secs(4), actions, None);
    }

    fn register_message(
        &mut self,
        _action: &Action<New, Event>,
    ) -> Result<Option<Box<dyn GameState>>, Error> {
        Err(Error::UnexpectedMessage)
    }

    fn register_response(
        &mut self,
        action: (Event, &Action<Received, Event>),
    ) -> Result<Option<Box<dyn GameState>>, Error> {
        // This state can only handle connected or ready checks
        let (response, request) = action;

        match request.action() {
            Event::ReadyCheck => match response {
                Event::Accept => {
                    self.ready.push(request.player() as u8);
                    let mut del_idx = None;
                    for (idx, player) in self.pending_ready.iter().enumerate() {
                        if *player == request.player() as u8 {
                            del_idx = Some(idx);
                        }
                    }
                    if let Some(idx) = del_idx {
                        self.pending_ready.remove(idx);
                    } else {
                        return Err(Error::UnexpectedResponse);
                    }
                }
                Event::Deny => {
                    let mut del_idx = None;
                    for (idx, player) in self.pending_ready.iter().enumerate() {
                        if *player == request.player() as u8 {
                            del_idx = Some(idx);
                        }
                    }
                    if let Some(idx) = del_idx {
                        self.pending_ready.remove(idx);
                    } else {
                        return Err(Error::UnexpectedResponse);
                    }
                }
                _ => {
                    return Err(Error::UnexpectedResponse);
                }
            },
            _ => {
                return Err(Error::UnexpectedResponse);
            }
        };
        Ok(None)
    }
    fn metadata(&mut self) -> Option<&mut GameMetaData> {
        None
    }
}
