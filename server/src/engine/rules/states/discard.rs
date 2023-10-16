use std::marker::PhantomData;

use crate::engine::rules::{GameMetaData, Action, New, Event, Error, Received};

use super::{GameState, PassHand, DiscardCard};


impl DiscardCard {
    pub fn new(state: GameMetaData) -> Self {
        Self {
            state,
            pending: Vec::new(),
            requested: false,
        }
    }
}

impl GameState for DiscardCard {
    fn get_next_action(
        &mut self,
        players: &Vec<usize>,
    ) -> (
        tokio::time::Duration,
        Vec<Action<New, Event>>,
        Option<Box<dyn GameState>>,
    ) {
        let mut actions = Vec::new();
        let mut request = |event: Event| {
            for player in players {
                if !self.pending.contains(&(*player as u8)) {
                    actions.push(Action {
                        player: *player,
                        action: event.clone(),
                        status: PhantomData,
                    })
                }
            }
        };
        // If we have any out standing messages await these
        if self.pending.len() != 0 {
            request(Event::WaitingForPlayers);
            // Sleep server for a long time since there is noting to do
            return (tokio::time::Duration::from_secs(2), actions, None);
        }
        if !self.requested {
            for player in players {
                actions.push(Action {
                    player: *player,
                    action: Event::DiscardRequest,
                    status: PhantomData,
                });
                self.pending.push(*player as u8);
            }
            self.requested = true;
        } else {
            return (
                tokio::time::Duration::from_secs(2),
                actions,
                Some(Box::new(PassHand::new(self.state.clone()))),
            );
        }
        (tokio::time::Duration::from_secs(2), actions, None)
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
        println!("{:?}", action);
        let (
            response,
            Action {
                player,
                status,
                action,
            },
        ) = action;
        println!("{:?},{:?},{:?}", response, player, action);

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
            Event::DiscardRequest => match response {
                Event::Discard(idx) => match self.state.discard(player, &idx) {
                    Ok(()) => {
                        self.pending.remove(request_idx);
                        Ok(None)
                    }
                    Err(e) => Err(e),
                },
                _ => Err(Error::UnexpectedResponse),
            },
            _ => Err(Error::UnexpectedResponse),
        }
    }
}