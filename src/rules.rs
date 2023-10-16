pub mod cards;
pub mod states;
use serde::{Deserialize, Serialize};
use server::engine::{
    event::{BackendEvent, GameEvent},
    rules::{Action, Completed, Error, Received, RuleEngine, Instantiable,New},
};

use self::{
    cards::{AustraliaCard, AustraliaDeck, AustralianActivities},
    states::{DealingCards, GameState, WaitingForPlayers},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Event {
    ReadyCheck,
    Accept,
    Deny,
    Deal(AustraliaCard),
    Hand(Vec<AustraliaCard>),
    ShowRequest,
    /// Shows the given card to the other players and discards it.
    Show(usize),
    ShowPile(u8, Vec<AustraliaCard>),
    DiscardRequest,
    /// Discards the card in the players hand at that given index.
    Discard(usize),
    ScoreActivityQuery(Vec<AustralianActivities>),
    ScoreActivity(Option<AustralianActivities>),
    ReassignHand(Vec<AustraliaCard>),
    WaitingForPlayers,
    Connected(u8),
    UnexpectedMessage,
}

impl Into<Vec<u8>> for Event {
    fn into(self) -> Vec<u8> {
        serde_json::to_string(&self).unwrap().into_bytes()
    }
}
impl From<BackendEvent> for Event {
    fn from(value: BackendEvent) -> Self {
        match value {
            BackendEvent::Connected(uid) => Self::Connected(uid),
            BackendEvent::UnexpectedMessage => Self::UnexpectedMessage,
        }
    }
}

impl GameEvent for Event {
    fn requires_response(&self) -> bool {
        match self {
            Event::ReadyCheck => true,
            Event::Deal(_) => true,
            Event::Hand(_) => true,
            Event::DiscardRequest => true,
            Event::ShowRequest => true,
            Event::ReassignHand(_) => true,
            Event::ScoreActivityQuery(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AustraliaPlayer {
    id: u8,
    hand: Vec<AustraliaCard>,
    discard_pile: Vec<AustraliaCard>,
    show_pile: Vec<AustraliaCard>,
    un_scored_activity: Vec<AustralianActivities>,
    #[allow(dead_code)]
    activity_scores: Vec<(AustralianActivities, usize)>,
}
#[derive(Debug, Clone)]
pub struct GameMetaData {
    deck: AustraliaDeck,
    players: Vec<AustraliaPlayer>,
}
impl AustraliaPlayer {
    fn new(id: u8) -> Self {
        Self {
            id: id,
            hand: Vec::new(),
            discard_pile: Vec::new(),
            show_pile: Vec::new(),
            un_scored_activity: AustralianActivities::to_vec(),
            activity_scores: Vec::new(),
        }
    }
    fn discard(&mut self, idx: &usize) -> Result<(), Error> {
        if *idx >= self.hand.len() {
            return Err(Error::NoSuchCard);
        }
        let card = self.hand.remove(*idx);
        self.discard_pile.push(card);
        Ok(())
    }
    fn show(&mut self, idx: &usize) -> Result<(), Error> {
        if *idx >= self.hand.len() {
            return Err(Error::NoSuchCard);
        }
        let card = self.hand.remove(*idx);
        self.show_pile.push(card);
        Ok(())
    }
    fn hand_empty(&self) -> bool {
        self.hand.len() == 0
    }
}

impl GameMetaData {
    const MAX_CARDS: usize = 7;
    fn new(players: &[usize]) -> Self {
        let mut players_vec = Vec::with_capacity(players.len());
        for player in players {
            players_vec.push(AustraliaPlayer::new(*player as u8));
        }
        Self {
            deck: AustraliaDeck::default(),
            players: players_vec,
        }
    }
    fn draft(&mut self) -> (bool, Vec<Action<New, Event>>) {
        let mut done = true;
        let mut actions = Vec::new();
        for player in self.players.iter_mut() {
            if player.hand.len() == Self::MAX_CARDS {
                continue;
            }
            done = false;
            let card = self.deck.draft();
            let action = Action::<New, Event>::new(player.id as usize, Event::Deal(card));
            player.hand.push(card);
            actions.push(action);
        }
        (done, actions)
    }
    fn discard(&mut self, uid: &usize, idx: &usize) -> Result<(), Error> {
        let mut selected_player = None;
        for player in &mut self.players {
            if player.id as usize == *uid {
                selected_player = Some(player);
                break;
            }
        }
        match selected_player {
            Some(player) => player.discard(idx),
            _ => return Err(Error::NoSuchCard),
        }
    }
    fn show(&mut self, uid: &usize, idx: &usize) -> Result<(), Error> {
        let mut selected_player = None;
        for player in &mut self.players {
            if player.id as usize == *uid {
                selected_player = Some(player);
                break;
            }
        }
        match selected_player {
            Some(player) => player.show(idx),
            _ => return Err(Error::NoSuchCard),
        }
    }
    /// Circulates the players hands in between them
    fn circulate(&mut self) {
        let mut prev_hand = self.players.last().unwrap().hand.clone();
        for player in self.players.iter_mut() {
            let intermediate = player.hand.clone();
            player.hand = prev_hand;
            prev_hand = intermediate;
        }
    }
    fn hands_empty(&self) -> bool {
        let mut empty = true;
        for player in &self.players {
            if !player.hand_empty() {
                empty = false;
            }
        }
        empty
    }
}

pub struct Australia<const CAPACITY: usize, const MIN_PLAYERS: usize> {
    state: Box<dyn GameState>,
}

impl<const CAPACITY: usize, const MIN_PLAYERS: usize> RuleEngine
    for Australia<CAPACITY, MIN_PLAYERS>
{
    type Event = Event;
    fn get_next_action(
        &mut self,
        players: &Vec<usize>,
    ) -> (tokio::time::Duration, Vec<Action<New, Self::Event>>) {
        let (delay, actions, state) = self.state.get_next_action(players);
        if let Some(state) = state {
            self.state = state;
        }

        (delay, actions)
    }

    fn register_response(
        &mut self,
        _: &Vec<usize>,
        response: (Self::Event, &Action<Received, Self::Event>),
    ) -> Result<Action<Completed, Self::Event>, Error> {
        let completed_action =
            Action::<Completed, Event>::new(response.1.player(), response.1.action().clone());
        let res = self.state.register_response(response);
        match res {
            Ok(val) => {
                match val {
                    Some(state) => self.state = state,
                    None => {}
                }
                Ok(completed_action)
            }
            Err(e) => Err(e),
        }
    }

    fn register_message(
        &mut self,
        _players: &Vec<usize>,
        _message: &Action<New, Self::Event>,
    ) -> Result<(), Error> {
        return Err(Error::UnexpectedMessage);
    }
}
impl<const CAPACITY: usize, const MIN_PLAYERS: usize> Instantiable
    for Australia<CAPACITY, MIN_PLAYERS>
{
    fn new() -> Self {
        Australia {
            state: Box::new(WaitingForPlayers::<DealingCards>::new(None)),
        }
    }
}
