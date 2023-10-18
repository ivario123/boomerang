pub mod cards;
pub mod states;
use std::path::MAIN_SEPARATOR;

use serde::{Deserialize, Serialize};
use server::engine::{
    event::{BackendEvent, GameEvent},
    rules::{Action, Completed, Error, Instantiable, New, Received, RuleEngine},
};

use crate::australia::mainpage::CardArea;

use self::{
    cards::{
        Animal, AustraliaCard, AustraliaDeck, AustralianActivity, AustralianAnimal,
        AustralianRegion, Card, Collection,
    },
    states::{pass::Direction, DealingCards, GameState, WaitingForPlayers},
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
    ShowPile(u8, Vec<AustraliaCard>,Vec<char>),
    DiscardRequest,
    /// Discards the card in the players hand at that given index.
    Discard(usize),
    ScoreActivityQuery(Vec<AustralianActivity>),
    ScoreActivity(Option<AustralianActivity>),
    ReassignHand(Vec<AustraliaCard>),
    WaitingForPlayers,
    WaitingForPlayer,
    Connected(u8),
    UnexpectedMessage,
    Resend,
    Sync(AustraliaPlayer),
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
            Event::Sync(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Scoring {
    throw_catch: usize,
    tourist_sites: usize,
    collections: usize,
    animals: usize,
    activity: usize,
    completed_regions: Vec<AustralianRegion>,
}

// Builder pattern for scoring
impl Scoring {
    fn new() -> Self {
        Self {
            throw_catch: 0,
            tourist_sites: 0,
            collections: 0,
            animals: 0,
            activity: 0,
            completed_regions: Vec::new(),
        }
    }

    fn completed_regions(&self) -> Vec<AustralianRegion> {
        self.completed_regions.clone()
    }

    fn score_throw_catch(mut self, player: &AustraliaPlayer) -> Self {
        let throw = player.get_discard()[0].number();
        let catch = player.get_hand()[0].number();
        self.throw_catch = {
            if throw > catch {
                throw - catch
            } else {
                catch - throw
            }
        };
        self
    }
    fn score_collections(mut self, player: &AustraliaPlayer) -> Self {
        let mut cards = player.get_discard();
        cards.extend(player.get_show());
        cards.extend(player.get_hand());
        let mut sum = 0;
        for card in cards {
            sum += match card.collection() {
                Some(collection) => collection.score(),
                _ => 0,
            };
        }
        self.collections = match sum > 7 {
            false => sum * 2,
            _ => sum,
        };
        self
    }

    fn score_animals(mut self, player: &AustraliaPlayer) -> Self {
        let mut map = std::collections::HashMap::<AustralianAnimal, bool>::new();
        let mut cards = player.get_discard();
        cards.extend(player.get_show());
        cards.extend(player.get_hand());
        let mut sum = 0;

        for card in cards {
            let animal = card.animal();
            if let Some(animal) = animal {
                match map.get(&animal) {
                    Some(value) => {
                        if *value {
                            sum += animal.score();
                        }

                        map.insert(animal, !value);
                    }
                    None => {
                        let _ = map.insert(animal, true);
                    }
                }
            }
        }
        self.animals = sum;
        self
    }
    fn score_regions(
        mut self,
        player: &AustraliaPlayer,
        unclaimed_region: &Vec<AustralianRegion>,
    ) -> Self {
        let mut cards = player.get_discard();
        cards.extend(player.get_show());
        cards.extend(player.get_hand());
        let mut visited = player.get_visited();
        let mut total = cards.len();
        for card in cards {
            // Insert if not exists
            if !visited.contains(&card.to_char()) {
                visited.push(card.to_char());
            }
        }
        let mut completed = Vec::new();
        for region in unclaimed_region {
            if region.completed(&visited) {
                completed.push(region.clone());
            }
        }
        total += completed.len() * 3;
        self.completed_regions = completed;
        self.tourist_sites = total;
        self
    }
    fn score_activity(
        mut self,
        player: &mut AustraliaPlayer,
        activity: Option<AustralianActivity>,
    ) -> Self {
        if let None = activity {
            return self;
        }
        let mut cards = player.get_discard();
        cards.extend(player.get_show());
        cards.extend(player.get_hand());
        let mut total = 0;
        if let Some(activity) = activity {
            let mut target_idx = None;
            for (idx, el) in player.un_scored_activity.iter().enumerate() {
                if *el == activity {
                    target_idx = Some(idx);
                    break;
                }
            }
            if let Some(idx) = target_idx {
                player.un_scored_activity.remove(idx);
            }
        }
        for card in cards {
            if card.activity() == activity {
                total += 1;
            }
        }
        self.activity += total;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AustraliaPlayer {
    id: u8,
    hand: Vec<AustraliaCard>,
    discard_pile: Vec<AustraliaCard>,
    show_pile: Vec<AustraliaCard>,
    un_scored_activity: Vec<AustralianActivity>,
    #[allow(dead_code)]
    activity_scores: Vec<(AustralianActivity, usize)>,
    card_ptr: usize,
    visited: Vec<char>,
    scoring: Vec<Scoring>,
}

#[derive(Debug, Clone)]
pub struct GameMetaData {
    deck: AustraliaDeck,
    players: Vec<AustraliaPlayer>,
    non_completed_regions: Vec<AustralianRegion>,
    round_counter: usize,
}

impl GameMetaData {
    /// Returns true if the game should end if not it returns false
    pub fn score_round(
        &mut self,
        score_activities: &Vec<(u8, Option<AustralianActivity>)>,
    ) -> bool {
        let mut completed = Vec::new();
        for player in &mut self.players {
            let mut activity = None;
            for (uid, score_activity) in score_activities {
                if player.id == *uid {
                    activity = score_activity.clone();
                    break;
                }
            }
            let scoring = Scoring::new()
                .score_throw_catch(player)
                .score_collections(player)
                .score_regions(player, &self.non_completed_regions)
                .score_activity(player, activity);

            for el in scoring.completed_regions() {
                if !completed.contains(&el) {
                    completed.push(el)
                }
            }
            player.add_score(scoring);
        }
        for el in completed {
            self.non_completed_regions.push(el);
        }
        self.round_counter == 3
    }
    pub fn new_round(&mut self) {
        self.deck = AustraliaDeck::default();
        self.round_counter += 1;
        for player in self.players.iter_mut() {
            player.new_round();
        }
    }
}

impl AustraliaPlayer {
    fn new(id: u8) -> Self {
        Self {
            id,
            hand: Vec::new(),
            discard_pile: Vec::new(),
            show_pile: Vec::new(),
            un_scored_activity: AustralianActivity::to_vec(),
            activity_scores: Vec::new(),
            card_ptr: 0,
            visited: Vec::new(),
            scoring: Vec::new(),
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

    pub fn privately_visited(&mut self) -> Vec<char> {
        let mut ret = self.publicly_visited();
        for el in self.get_discard() {
            if !ret.contains(&el.to_char()) {
                ret.push(el.to_char());
            }
        }
        ret
    }

    pub fn publicly_visited(&self) -> Vec<char> {
        let cards = self.get_show();
        let mut ret = self.get_visited();
        for el in cards {
            if !ret.contains(&el.to_char()) {
                ret.push(el.to_char());
            }
        }
        ret
    }

    pub fn new_round(&mut self) {
        let mut cards = self.get_discard();
        cards.extend(self.get_show());
        cards.extend(self.get_hand());

        for card in cards {
            if !self.visited.contains(&card.to_char()) {
                self.visit(card.to_char());
            }
        }

        self.hand.clear();
        self.discard_pile.clear();
        self.show_pile.clear();
    }
    pub fn add_score(&mut self, score: Scoring) {
        self.scoring.push(score);
    }
    pub fn visit(&mut self, site: char) {
        self.visited.push(site);
    }
    pub fn get_visited(&self) -> Vec<char> {
        self.visited.clone()
    }
    pub fn get_hand(&self) -> Vec<AustraliaCard> {
        self.hand.clone()
    }
    pub fn get_discard(&self) -> Vec<AustraliaCard> {
        self.discard_pile.clone()
    }
    pub fn get_show(&self) -> Vec<AustraliaCard> {
        self.show_pile.clone()
    }

    pub fn set_cards(mut self, cards: Vec<AustraliaCard>) -> Self {
        self.hand = cards;
        self
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
            un_scored_activity: AustralianActivity::to_vec(),
            activity_scores: Vec::new(),
            card_ptr: 0,
            visited: Vec::new(),
            scoring: Vec::new(),
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
            non_completed_regions: AustralianRegion::to_vec(),
            round_counter: 0,
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
    fn circulate(&mut self, direction: Direction) {
        let players = match direction {
            Direction::Forward => {
                let mut prev_hand = self.players.last().unwrap().hand.clone();
                for player in self.players.iter_mut() {
                    let intermediate = player.hand.clone();
                    player.hand = prev_hand;
                    prev_hand = intermediate;
                }
            }
            Direction::Backward => {
                let mut prev_hand = self.players.last().unwrap().hand.clone();
                for player in self.players.iter_mut() {
                    let intermediate = player.hand.clone();
                    player.hand = prev_hand;
                    prev_hand = intermediate;
                }
            }
        };
    }
    fn hands_singleton(&self) -> bool {
        let mut ret = true;
        for player in &self.players {
            if player.hand_size() != 1 {
                ret = false;
            }
        }
        ret
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
