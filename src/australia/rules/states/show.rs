use log::info;

use crate::{
    engine::rules::{Action, Error, New, Received}, australia::{rules::meta::GameMetaData, protocol::Event},
};

use super::{pass::Direction, GameState, PassHand, AsMetaData, Scoring, ShowCard};

impl ShowCard {
    pub fn new(state: GameMetaData) -> Self {
        Self {
            state,
            pending: Vec::new(),
            requested: false,
        }
    }
}

impl From<GameMetaData> for ShowCard {
    fn from(value: GameMetaData) -> Self {
        Self::new(value)
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
        info!("State : {:?}", self);
        let mut actions = Vec::new();
        let mut request = |event: Event| {
            for player in players {
                if !self.pending.contains(&(*player as u8)) {
                    actions.push(Action::new(*player, event.clone()))
                }
            }
        };
        // If we have any out standing messages await them
        if self.pending.len() != 0 {
            request(Event::WaitingForPlayers);
            // Sleep server for a long time since there is noting to do
            return (tokio::time::Duration::from_millis(500), actions, None);
        }
        if !self.requested {
            for player in self.state.get_players() {
                actions.push(Action::new(player.id as usize, Event::ShowRequest));
                self.pending.push(player.id);
            }
            self.requested = true;
        } else if self.requested && self.pending.len() == 0 {
            // Show every other players show pile to the others and transition
            for player in self.state.get_players().clone() {
                for other_player in self.state.get_players() {
                    if player.id != other_player.id {
                        info!(
                            "Sending player {:?}s hand to player {:?}",
                            player.id, other_player.id
                        );
                        actions.push(Action::new(
                            other_player.id as usize,
                            Event::ShowPile(
                                player.id,
                                player.show_pile.clone(),
                                player.publicly_visited(),
                            ),
                        ));
                    }
                }
            }
            if self.state.hands_singleton() {
                // Now we move to scoring
                return (
                    tokio::time::Duration::from_millis(500),
                    actions,
                    Some(Box::new(PassHand::<Scoring>::new(
                        self.state.clone(),
                        Direction::Backward,
                    ))),
                );
            } else {
                return (
                    tokio::time::Duration::from_millis(500),
                    actions,
                    Some(Box::new(PassHand::<ShowCard>::new(
                        self.state.clone(),
                        Direction::Forward,
                    ))),
                );
            }
        }
        (tokio::time::Duration::from_millis(500), actions, None)
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
        let (response, action) = action;
        let (player, action) = (action.player(), action.action());

        println!("{:?},{:?},{:?}", response, player, action);

        let mut outstanding_request = None;
        for (idx, &id) in self.pending.iter().enumerate() {
            if id as usize == player {
                outstanding_request = Some(idx);
            }
        }
        let request_idx = match outstanding_request {
            Some(idx) => idx,
            _ => return Err(Error::UnexpectedResponse),
        };

        match action {
            Event::ShowRequest => match response {
                Event::Show(idx) => match self.state.show(&player, &idx) {
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
    fn metadata(&mut self) -> Option<&mut GameMetaData> {
        Some(AsMetaData::metadata(self))
    }
}
