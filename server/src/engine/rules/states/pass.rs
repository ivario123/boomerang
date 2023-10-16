use std::marker::PhantomData;

use crate::engine::rules::{Action, Error, Event, GameMetaData, New, Received};

use super::{GameState,PassHand, ShowCard};

impl PassHand {
    pub fn new(state: GameMetaData) -> Self {
        Self {
            state,
            pending: Vec::new(),
            requested: false,
        }
    }
}

impl GameState for PassHand {
    fn get_next_action(
        &mut self,
        players: &Vec<usize>,
    ) -> (
        tokio::time::Duration,
        Vec<Action<New, Event>>,
        Option<Box<dyn GameState>>,
    ) {
        let mut actions = Vec::new();
        // If we have any out standing messages await these
        if self.pending.len() != 0 {
            for player in players {
                if !self.pending.contains(&(*player as u8)) {
                    actions.push(Action {
                        player: *player,
                        action: Event::WaitingForPlayers,
                        status: PhantomData,
                    })
                }
            }
            // Sleep server for a long time since there is noting to do
            return (tokio::time::Duration::from_secs(20), actions, None);
        }
        self.state.circulate();
        if !self.requested {
            for player in &mut self.state.players {
                actions.push(Action {
                    player: player.id as usize,
                    action: Event::ReassignHand(player.hand.clone()),
                    status: PhantomData,
                });
                self.pending.push(player.id);
            }
            self.requested = true;
            (tokio::time::Duration::from_secs(1), actions, None)
        } else {
            (
                tokio::time::Duration::from_secs(1),
                actions,
                Some(Box::new(ShowCard::new(self.state.clone()))),
            )
        }
    }

    fn register_message(
        &mut self,
        action: &Action<New, Event>,
    ) -> Result<Option<Box<dyn GameState>>, Error> {
        todo!()
    }

    fn register_response(
        &mut self,
        action: (Event, &Action<Received, Event>),
    ) -> Result<Option<Box<dyn GameState>>, Error> {
        let (
            response,
            Action {
                player,
                status,
                action,
            },
        ) = action;

        let mut outstanding_request = None;
        for (idx, &id) in self.pending.iter().enumerate() {
            if id as usize == *player {
                outstanding_request = Some(idx);
            }
        }
        let request_idx = match outstanding_request {
            Some(idx) => idx,
            _ => return Err(Error::UnexpectedResponse),
        };

        match action {
            Event::ReassignHand(_) => match response {
                Event::Accept => {
                    self.pending.remove(request_idx);
                    Ok(None)
                }
                _ => Err(Error::UnexpectedResponse),
            },
            _ => Ok(None),
        }
    }
}
