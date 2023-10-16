use super::{Action, Error, Event, GameMetaData, New, Received};

pub mod dealing;
pub mod discard;
pub mod pass;
pub mod score;
pub mod show;
pub mod waiting;

pub trait GameState: Send + std::fmt::Debug {
    fn get_next_action(
        &mut self,
        players: &Vec<usize>,
    ) -> (
        tokio::time::Duration,
        Vec<Action<New, Event>>,
        Option<Box<dyn GameState>>,
    );
    fn register_message(
        &mut self,
        action: &Action<New, Event>,
    ) -> Result<Option<Box<dyn GameState>>, Error>;
    fn register_response(
        &mut self,
        action: (Event, &Action<Received, Event>),
    ) -> Result<Option<Box<dyn GameState>>, Error>;
}

#[derive(Debug)]
pub struct DealingCards {
    state: GameMetaData,
    pending_actions: Vec<u8>,
    #[allow(dead_code)]
    validated: Vec<usize>,
}
#[derive(Debug)]
pub struct WaitingForPlayers<Next: GameState + Send> {
    ready: Vec<u8>,
    pending_ready: Vec<u8>,
    next_state: Option<Box<Next>>,
}

#[derive(Debug)]
pub struct DiscardCard {
    state: GameMetaData,
    pending: Vec<u8>,
    requested: bool,
}
#[derive(Debug)]
pub struct PassHand {
    state: GameMetaData,
    pending: Vec<u8>,
    requested: bool,
}

#[derive(Debug)]
pub struct ShowCard {
    state: GameMetaData,
    pending: Vec<u8>,
    requested: bool,
}

#[derive(Debug)]
pub struct Scoring {
    state: GameMetaData,
    pending: Vec<u8>,
    requested: bool,
}
