use std::marker::PhantomData;

use log::info;

use crate::{
    engine::rules::{Action, Error, New, Received},
    rules::{Event, GameMetaData},
};

use super::{GameState, PassHand, ShowCard, Syncing, ReprMetaData};

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Forward,
    Backward,
}

impl<Next: ReprMetaData + Send + Sync + From<GameMetaData>> PassHand<Next> {
    pub fn new(state: GameMetaData, direction: Direction) -> Self {
        Self {
            state,
            pending: Vec::new(),
            requested: false,
            direction: direction,
            next: PhantomData,
        }
    }
}

impl<Next: ReprMetaData + Send + Sync + From<GameMetaData> + 'static> GameState for PassHand<Next> {
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
        // If we have any out standing messages await these
        if self.pending.len() != 0 {
            for player in players {
                if !self.pending.contains(&(*player as u8)) {
                    actions.push(Action::new(*player, Event::WaitingForPlayers))
                }
            }
            // Sleep server for a long time since there is noting to do
            return (tokio::time::Duration::from_secs(20), actions, None);
        }
        if !self.requested {
            self.state.circulate(self.direction);
            for player in &mut self.state.players {
                actions.push(Action::new(
                    player.id as usize,
                    Event::ReassignHand(player.hand.clone()),
                ));
                self.pending.push(player.id);
            }
            self.requested = true;
            return (tokio::time::Duration::from_secs(1), actions, None);
        }
        (
            tokio::time::Duration::from_secs(1),
            actions,
            Some(Box::new(Syncing::new(
                self.state.clone(),
                Box::new(Next::from(self.state.clone())),
            ))),
        )
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
    fn metadata(&mut self) -> Option<&mut GameMetaData> {
        Some(ReprMetaData::metadata(self))
    }
}
