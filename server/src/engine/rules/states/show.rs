use std::marker::PhantomData;

use crate::engine::rules::{Action, Error, Event, GameMetaData, New, Received};

use super::{GameState, PassHand, ShowCard, Scoring};

impl ShowCard {
    pub fn new(state: GameMetaData) -> Self {
        Self {
            state,
            pending: Vec::new(),
            requested: false,
        }
    }
}

impl GameState for ShowCard {
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
        // If we have any out standing messages await them
        if self.pending.len() != 0 {
            request(Event::WaitingForPlayers);
            // Sleep server for a long time since there is noting to do
            return (tokio::time::Duration::from_secs(2), actions, None);
        }
        if !self.requested {
            for player in &mut self.state.players {
                actions.push(Action {
                    player: player.id as usize,
                    action: Event::ShowRequest,
                    status: PhantomData,
                });
                self.pending.push(player.id);
            }
            self.requested = true;
        } else if self.requested && self.pending.len() == 0 {
            // Show every other players show pile to the others and transition
            for player in &self.state.players {
                for other_player in &self.state.players {
                    if player.id != other_player.id {
                        actions.push(Action {
                            player: other_player.id as usize,
                            action: Event::ShowPile(player.id, player.show_pile.clone()),
                            status: PhantomData,
                        });
                    }
                }
            }
            if self.state.hands_empty() {
                // Now we move to scoring
                return (
                    tokio::time::Duration::from_secs(2),
                    actions,
                    Some(Box::new(Scoring::new(self.state.clone()))),
                );
            } else {
                return (
                    tokio::time::Duration::from_secs(2),
                    actions,
                    Some(Box::new(PassHand::new(self.state.clone()))),
                );
            }
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
        _action: &Action<New, Event>,
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
                action,
                ..
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
            Event::ShowRequest => match response {
                Event::Show(idx) => match self.state.show(player, &idx) {
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
