use std::{cell::RefCell, marker::PhantomData};

use self::{states::{DealingCards, WaitingForPlayers, GameState}, cards::{AustraliaCard, AustralianActivities, AustraliaDeck}};

use super::{
    event::{self, BackendEvent, GameEvent},
    player::{self, Player},
    session::Session,
};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::iter::FromIterator;

pub mod states;
pub trait ActionStatus {}
pub mod cards;

macro_rules! impl_status {
    (New) => {
        impl<Event:GameEvent> Action<New,Event>{
            fn from_action<Status:ActionStatus>(action:Action<Status,Event>) -> Self{
                Self{
                    player:action.player,
                    action:action.action,
                    status:PhantomData
                }
            }
        }
    };
    ($status:ident) => {
        impl<Event:GameEvent> Into<Action<New,Event>> for Action<$status,Event>{
            fn into(self) -> Action<New,Event> {
                Action::<New,Event>::from_action(self)
            }
        }
    };
    (all $($status:ident)+) => {
        $(
            impl ActionStatus for $status {}
            impl_status!($status);
        )+
    };
}
macro_rules! transition_of_status {
    ($($status1:ident -> $status2:ident)+) => {
        $(
            impl<Event:GameEvent>  Action<$status1,Event> {
                pub fn transition(self) -> Action<$status2,Event>{
                    Action::<$status2,Event>{
                        player:self.player,
                        action:self.action,
                        status:PhantomData
                    }
                }
            }
            impl<Event:GameEvent>  Action<$status2,Event> {
                pub fn degrade(self) -> Action<$status1,Event>{
                    Action::<$status1,Event>{
                        player:self.player,
                        action:self.action,
                        status:PhantomData
                    }
                }
            }
        )+
    };
}
#[derive(Debug)]
pub struct New {}
#[derive(Debug)]
pub struct Sent {}
#[derive(Debug)]
pub struct Received {}
#[derive(Debug)]
pub struct Completed {}
impl_status!(all New Sent Received Completed);
transition_of_status!(New -> Sent Sent -> Received Received -> Completed);

#[derive(Debug)]
pub struct Action<Status: ActionStatus, Event: GameEvent> {
    player: usize,
    action: Event,
    status: PhantomData<Status>,
}

impl<Status: ActionStatus, Event: GameEvent> Action<Status, Event> {
    pub fn player(&self) -> usize {
        self.player
    }
    pub fn action(&self) -> Event {
        self.action.clone()
    }
    pub fn new(player: usize, action: Event) -> Action<New, Event> {
        Action {
            player,
            action,
            status: PhantomData,
        }
    }
}
#[derive(Debug)]
pub enum Error {
    /// Thrown when the response recieved is not the expected one.
    ///
    /// Wraps the requested action to resolve this and the event that triggered the error
    UnexpectedResponse,
    /// Thrown when a un prompted un expected event occurs.
    UnexpectedMessage,
    /// Thrown when the selected card is out of range
    NoSuchCard,
}

pub trait RuleEngine {
    type Event: event::GameEvent + Send;

    /// Returns the next set of actions, this could be 1 action or it could be many.
    /// Also it returns the minimum time to wait before requesting any new actions,
    /// this is use full if the game is in a wait state, say that not all players have
    /// selected a throw card.
    fn get_next_action(
        &mut self,
        players: &Vec<usize>,
    ) -> (tokio::time::Duration, Vec<Action<New, Self::Event>>);
    fn register_response(
        &mut self,
        players: &Vec<usize>,
        response: (Self::Event, &Action<Received, Self::Event>),
    ) -> Result<Action<Completed, Self::Event>, Error>;
    fn register_message(
        &mut self,
        players: &Vec<usize>,
        message: &Action<New, Self::Event>,
    ) -> Result<(), Error>;
}

pub trait Instantiable {
    fn new() -> Self;
}

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
    scorable_activity: Vec<AustralianActivities>,
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
            scorable_activity: AustralianActivities::to_vec(),
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
    const MAXCARDS: usize = 7;
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
    fn hands<'a>(&'a mut self) -> &'a mut Vec<AustraliaPlayer> {
        &mut self.players
    }
    fn draft(&mut self) -> (bool, Vec<Action<New, Event>>) {
        let mut done = true;
        let mut actions = Vec::new();
        for player in self.players.iter_mut() {
            if player.hand.len() == 7 {
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


pub struct Austrailia<const CAPACITY: usize, const MIN_PLAYERS: usize> {
    state: Box<dyn GameState>,
}

impl<const CAPACITY: usize, const MIN_PLAYERS: usize> RuleEngine
    for Austrailia<CAPACITY, MIN_PLAYERS>
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
        players: &Vec<usize>,
        response: (Self::Event, &Action<Received, Self::Event>),
    ) -> Result<Action<Completed, Self::Event>, Error> {
        let completed_action = Action::<Completed, Event> {
            player: response.1.player,
            action: response.1.action.clone(),
            status: PhantomData,
        };
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
        players: &Vec<usize>,
        message: &Action<New, Self::Event>,
    ) -> Result<(), Error> {
        return Err(Error::UnexpectedMessage);
    }
}
impl<const CAPACITY: usize, const MIN_PLAYERS: usize> Instantiable
    for Austrailia<CAPACITY, MIN_PLAYERS>
{
    fn new() -> Self {
        Austrailia {
            state: Box::new(WaitingForPlayers::<DealingCards>::new(None)),
        }
    }
}
