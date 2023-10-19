use log::info;

use crate::{
    australia::{
        protocol::Event,
        rules::{meta::GameMetaData, states::Final},
    },
    engine::rules::{Action, Error, New, Received},
};

use super::{AsMetaData, DealingCards, GameState, Scoring, Syncing};

impl Scoring {
    pub fn new(state: GameMetaData) -> Self {
        Self {
            state,
            pending: Vec::new(),
            requested: false,
            actions: Vec::new(),
        }
    }
}

impl From<GameMetaData> for Scoring {
    fn from(value: GameMetaData) -> Self {
        Self::new(value)
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
            for player in self.state.get_players() {
                actions.push(Action::new(
                    player.id as usize,
                    Event::ScoreActivityQuery(player.un_scored_activity.clone()),
                ));
                self.pending.push(player.id);
            }
            self.requested = true;
            (tokio::time::Duration::from_secs(1), actions, None)
        } else {
            match self.state.score_round(&self.actions) {
                // Final state, this means game is over
                true => (
                    tokio::time::Duration::from_secs(1),
                    actions,
                    Some(Box::new(Final::from(self.state.clone()))),
                ),
                false => {
                    for player in self.state.get_players() {
                        actions.push(Action::new(player.id.into(), Event::NewRound));
                    }
                    self.state.new_round();
                    (
                        tokio::time::Duration::from_secs(1),
                        actions,
                        Some(Box::new(Syncing::new(
                            self.state.clone(),
                            Box::new(DealingCards::from(self.state.clone())),
                        ))),
                    )
                }
            }
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
            Event::ScoreActivityQuery(activities) => {
                // Here we should have an ok of some sort
                match response {
                    Event::ScoreActivity(activity) => {
                        match activity {
                            Some(activity) => {
                                if activities.contains(&activity) {
                                    self.actions.push((player as u8, Some(activity)));
                                } else {
                                    return Err(Error::UnexpectedMessage);
                                }
                            }
                            _ => {
                                self.actions.push((player as u8, None));
                            }
                        }
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
        Some(AsMetaData::metadata(self))
    }
}
