use log::info;

use crate::{
    engine::rules::{Action, Error, New, Received}, australia::{rules::meta::GameMetaData, protocol::Event},
};

use super::{GameState, AsMetaData, Syncing};

impl<Next: AsMetaData + Send + Sync> Syncing<Next> {
    pub fn new(state: GameMetaData, next_state: Box<Next>) -> Self {
        Self {
            state,
            pending: Vec::new(),
            requested: false,
            next_state: Some(next_state),
        }
    }
}
impl<Next: AsMetaData + Send + Sync + 'static> GameState for Syncing<Next> {
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
        info!("Syncing game state with {:?} pending events", self.pending);
        // If we have any out standing messages await these
        if self.pending.len() != 0 {
            // The response to this request is automated so it will be done fast
            return (tokio::time::Duration::from_millis(1), actions, None);
        }
        if !self.requested {
            for player in self.state.get_players() {
                actions.push(Action::new(player.id as usize, Event::Sync(player.clone())));
                self.pending.push(player.id);
            }
            self.requested = true;
            (tokio::time::Duration::from_millis(1), actions, None)
        } else {
            // Usage of unwrap here is intended, The only time this goes to none is when some
            // logic error has occurred, if it does, we should panic
            let state = std::mem::replace(&mut self.next_state, None).unwrap();
            (tokio::time::Duration::from_millis(1), actions, Some(state))
        }
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
        let (response, action) = action;
        let (player, action) = (action.player(), action.action());

        let mut pending = None;
        // The only responses that are valid in this context
        // is the
        for (idx, request) in self.pending.iter().enumerate() {
            if *request == player as u8 {
                pending = Some(idx);
                break;
            }
        }
        let pending = match pending {
            Some(idx) => idx,
            _ => return Err(Error::UnexpectedResponse),
        };

        match action {
            Event::Sync(_) => {
                // Here we should have an ok of some sort
                match response {
                    Event::Accept => {
                        self.pending.remove(pending);
                        Ok(None)
                    }
                    _ => Err(Error::UnexpectedResponse),
                }
            }
            _ => Err(Error::UnexpectedMessage),
        }
    }
    fn metadata(&mut self) -> Option<&mut GameMetaData> {
        Some(&mut self.state)
    }
}
