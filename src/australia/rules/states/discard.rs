use log::info;

use crate::{
    australia::{protocol::Event, rules::meta::GameMetaData},
    engine::rules::{Action, Error, New, Received},
};

use super::{pass::Direction, AsMetaData, DiscardCard, GameState, PassHand, ShowCard};

impl DiscardCard {
    pub fn new(state: GameMetaData) -> Self {
        Self {
            state,
            pending: Vec::new(),
            requested: false,
        }
    }
}

impl From<GameMetaData> for DiscardCard {
    fn from(metadata: GameMetaData) -> Self {
        Self::new(metadata)
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
        info!("State : {:?}", self);
        let mut actions = Vec::new();
        let mut request = |event: Event| {
            for player in players {
                if !self.pending.contains(&(*player as u8)) {
                    actions.push(Action::new(*player, event.clone()));
                }
            }
        };
        // If we have any out standing messages await these
        if self.pending.len() != 0 {
            request(Event::WaitingForPlayers);
            // Sleep server for a long time since there is noting to do
            return (tokio::time::Duration::from_millis(500), actions, None);
        }
        if !self.requested {
            for player in players {
                actions.push(Action::new(*player, Event::DiscardRequest));
                self.pending.push(*player as u8);
            }
            self.requested = true;
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
            Event::DiscardRequest => match response {
                Event::Discard(idx) => match self.state.discard(&player, &idx) {
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
