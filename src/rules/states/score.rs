use crate::{engine::rules::{Action, New, Error, Received}, rules::{GameMetaData, Event}};

use super::{Scoring, GameState};


impl Scoring {
    pub fn new(state: GameMetaData) -> Self {
        Self {
            state,
            pending: Vec::new(),
            requested: false,
        }
    }
}
impl GameState for Scoring {
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
                    actions.push(Action::new(*player,Event::WaitingForPlayers))
                }
            }
            // Sleep server for a long time since there is noting to do
            return (tokio::time::Duration::from_secs(20), actions, None);
        }
        if !self.requested {
            for player in &mut self.state.players {
                actions.push(Action::new(player.id as usize,Event::ScoreActivityQuery(player.un_scored_activity.clone())));
                self.pending.push(player.id);
            }
            self.requested = true;
            (tokio::time::Duration::from_secs(1), actions, None)
        } else {
            todo!();
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
        _action: (Event, &Action<Received, Event>),
    ) -> Result<Option<Box<dyn GameState>>, Error> {
        todo!()
    }
}