use crate::{
    engine::rules::{Action, Error, New, Received},
    rules::{Event, GameMetaData},
};

use super::{DealingCards, DiscardCard, GameState};

impl DealingCards {
    pub fn new(players: &[usize]) -> Self {
        Self {
            pending_actions: Vec::with_capacity(players.len()),
            validated: Vec::new(),
            state: GameMetaData::new(players),
        }
    }
}

impl GameState for DealingCards {
    fn get_next_action<'a>(
        &'a mut self,
        players: &Vec<usize>,
    ) -> (
        tokio::time::Duration,
        Vec<Action<New, Event>>,
        Option<Box<dyn GameState>>,
    ) {
        println!("In dealing state");
        let mut actions = Vec::new();

        // If we have any out standing messages await these
        if self.pending_actions.len() != 0 {
            for player in players {
                if !self.pending_actions.contains(&(*player as u8)) {
                    actions.push(Action::new(*player, Event::WaitingForPlayers));
                }
            }
            // Sleep server for a long time since there is noting to do
            return (tokio::time::Duration::from_secs(1), actions, None);
        }
        let (done, actions) = self.state.draft();
        for action in &actions {
            self.pending_actions.push(action.player() as u8)
        }
        if done {
            // Now we can transition to the discarding cards
            return (
                tokio::time::Duration::from_secs(1),
                actions,
                Some(Box::new(DiscardCard::new(self.state.clone()))),
            );
        }

        (tokio::time::Duration::from_secs(1), actions, None)
    }

    fn register_message(
        &mut self,
        _action: &Action<New, Event>,
    ) -> Result<Option<Box<dyn GameState>>, Error> {
        // Check if that player has an outstanding action
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
        for (idx, request) in self.pending_actions.iter().enumerate() {
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
            Event::Deal(_card) => {
                // Here we should have an ok of some sort
                match response {
                    Event::Accept => {
                        self.pending_actions.remove(pending);
                        Ok(None)
                    }
                    _ => Err(Error::UnexpectedResponse),
                }
            }
            _ => Err(Error::UnexpectedMessage),
        }
    }
}
