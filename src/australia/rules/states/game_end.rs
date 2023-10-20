use log::info;

use crate::{
    engine::rules::{Action, Error, New, Received}, australia::{rules::meta::GameMetaData, protocol::Event},
};

use super::{Final, GameState, AsMetaData};

impl Final {
    pub fn new(state: GameMetaData) -> Self {
        Self {
            state,
            delivered: false,
        }
    }
}

impl From<GameMetaData> for Final {
    fn from(value: GameMetaData) -> Self {
        Self::new(value)
    }
}

impl GameState for Final {
    fn get_next_action(
        &mut self,
        _players: &Vec<usize>,
    ) -> (
        tokio::time::Duration,
        Vec<Action<New, Event>>,
        Option<Box<dyn GameState>>,
    ) {
        info!("State : {:?}", self);
        let mut actions = Vec::new();
        if !self.delivered {
            let result = self.state.rank();
            for player in self.state.get_players() {
                actions.push(Action::new(
                    player.id as usize,
                    Event::FinalResult(player.id, result.clone()),
                ));
            }
            self.delivered = true;
            (tokio::time::Duration::from_millis(500), actions, None)
        } else {
            (tokio::time::Duration::from_secs(100), actions, None)
        }
    }

    fn register_message(
        &mut self,
        _action: &Action<New, Event>,
    ) -> Result<Option<Box<dyn GameState>>, Error> {
        Err(Error::UnexpectedMessage)
    }

    fn register_response(
        &mut self,
        _action: (Event, &Action<Received, Event>),
    ) -> Result<Option<Box<dyn GameState>>, Error> {
        Err(Error::UnexpectedResponse)
    }
    fn metadata(&mut self) -> Option<&mut GameMetaData> {
        Some(AsMetaData::metadata(self))
    }
}
