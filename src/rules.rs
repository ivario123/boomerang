pub mod cards;
pub mod states;
use serde::{Deserialize, Serialize};
use server::engine::{
    event::{BackendEvent, GameEvent},
    rules::{Action, Completed, Error, Instantiable, New, Received, RuleEngine},
};

use crate::australia::mainpage::CardArea;

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
    WaitingForPlayer,
    Connected(u8),
    UnexpectedMessage,
    Resend,
}
impl TryInto<BackendEvent> for Event {
    type Error = ();
    fn try_into(self) -> Result<BackendEvent, Self::Error> {
        match self {
            Self::Connected(uid) => Ok(BackendEvent::Connected(uid)),
            Self::UnexpectedMessage => Ok(BackendEvent::UnexpectedMessage),
            Self::Resend => Ok(BackendEvent::Resend),
            _ => Err(()),
        }
    }
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
            BackendEvent::Resend => Event::Resend,
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
    card_ptr: usize,
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
            card_ptr: 0,
        }
    }
    fn discard(&mut self, idx: &usize) -> Result<AustraliaCard, Error> {
        if *idx >= self.hand.len() {
            return Err(Error::NoSuchCard);
        }
        let card = self.hand.remove(*idx);
        self.discard_pile.push(card.clone());
        self.decrement();
        Ok(card)
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

    pub fn card_ptr(&mut self) -> &mut usize {
        &mut self.card_ptr
    }

    pub fn get_cards<const COUNT: usize>(
        &self,
        start: usize,
    ) -> (&[AustraliaCard], (usize, usize)) {
        if self.hand_empty() {
            return (&[], (0, 0));
        }
        match start > self.hand.len() - 1 {
            true => self.hand.len() - 1,
            false => start,
        };
        let end = match (start + COUNT) > self.hand.len() {
            false => start + COUNT,
            true => self.hand.len(),
        };
        (&self.hand[start..end], (end, self.hand.len()))
    }
    pub fn hand_size(&self) -> usize {
        self.hand.len()
    }
}

impl tui::ui::UiElement for AustraliaPlayer {
    /// This should never be called
    fn new() -> Self {
        Self {
            id: 0,
            hand: Vec::new(),
            discard_pile: Vec::new(),
            show_pile: Vec::new(),
            un_scored_activity: AustralianActivities::to_vec(),
            activity_scores: Vec::new(),
            card_ptr: 0,
        }
    }
}

impl tui::ui::Hand<AustraliaCard> for AustraliaPlayer {
    fn get<const COUNT: usize>(&self, start: usize) -> (&[AustraliaCard], (usize, usize)) {
        self.get_cards::<COUNT>(start)
    }

    fn count(&self) -> usize {
        self.hand_size()
    }

    fn add_card(&mut self, card: AustraliaCard) {
        if !self.hand.contains(&card) {
            self.hand.push(card);
        }
    }
    fn discard_card(&mut self, idx: usize) -> AustraliaCard {
        self.hand.remove(idx)
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
            Some(player) => {
                player.discard(idx)?;
                Ok(())
            }
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
