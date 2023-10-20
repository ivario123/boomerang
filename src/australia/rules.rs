pub mod cards;
pub mod meta;
pub mod scoring;
pub mod states;

use serde::{Deserialize, Serialize};
use server::engine::rules::{Action, Completed, Error, Instantiable, New, Received, RuleEngine};

use self::{
    cards::{AustraliaCard, AustralianActivity, Card},
    scoring::Scoring,
    states::{DealingCards, GameState, WaitingForPlayers},
};

use super::{protocol::Event, tui::pages::main_page::CardArea};

/// Player abstraction holds all relevant game data for a specific player.
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
impl AustraliaPlayer {
    pub fn new(id: u8) -> Self {
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
}
// Modifiers
impl AustraliaPlayer {
    /// Moves the card from that index to the discard pile
    ///
    /// Returns error if that index is invalid
    pub fn discard(&mut self, idx: &usize) -> Result<AustraliaCard, Error> {
        if *idx >= self.hand.len() {
            return Err(Error::NoSuchCard);
        }
        let card = self.hand.remove(*idx);
        self.discard_pile.push(card.clone());
        self.decrement();
        Ok(card)
    }
    /// Moves the card from that index to the show pile
    ///
    /// Returns error if that index is invalid
    fn show(&mut self, idx: &usize) -> Result<(), Error> {
        if *idx >= self.hand.len() {
            return Err(Error::NoSuchCard);
        }
        let card = self.hand.remove(*idx);
        self.show_pile.push(card);
        Ok(())
    }

    /// Prepares the player for a new round
    ///
    /// - Updates the list of visited locations
    /// - Clears the hand
    /// - Clears the discard pile
    /// - Clears the show pile
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

    /// Overwrites the cards in the players hand with a new list of cards
    pub fn set_cards(mut self, cards: Vec<AustraliaCard>) -> Self {
        self.hand = cards;
        self
    }
}
// Getters
impl AustraliaPlayer {
    /// Returns true if the players hand is empty
    fn hand_empty(&self) -> bool {
        self.hand.len() == 0
    }
    /// Returns a list of the players round scores
    pub fn scores(&self) -> Vec<Scoring> {
        self.scoring.clone()
    }
    /// Returns a list of all the places that the player
    /// has visited
    ///
    /// This includes the current discarded card.
    pub fn privately_visited(&mut self) -> Vec<char> {
        let mut ret = self.publicly_visited();
        for el in self.get_discard() {
            if !ret.contains(&el.to_char()) {
                ret.push(el.to_char());
            }
        }
        ret
    }

    /// Returns a list of all of the places that the player
    /// has visited aside from the currently discarded card
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

    /// Pushes a new score to the players list of scores
    pub fn add_score(&mut self, score: Scoring) {
        self.scoring.push(score);
    }

    /// pushes a new site to the places that a player has visited
    pub fn visit(&mut self, site: char) {
        self.visited.push(site);
    }

    /// Returns a clone of the sites that the player has visited
    pub fn get_visited(&self) -> Vec<char> {
        self.visited.clone()
    }

    /// Returns a clone of the cards on the players hand
    pub fn get_hand(&self) -> Vec<AustraliaCard> {
        self.hand.clone()
    }

    /// Returns a clone of the cards in the players discard pile
    pub fn get_discard(&self) -> Vec<AustraliaCard> {
        self.discard_pile.clone()
    }

    /// Returns a clone of the cards in the players show pile
    pub fn get_show(&self) -> Vec<AustraliaCard> {
        self.show_pile.clone()
    }
}
// Strictly TUI helpers
impl AustraliaPlayer {
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
            Action::<Completed, Self::Event>::new(response.1.player(), response.1.action().clone());
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

#[cfg(test)]
mod tests {
    use tui::ui::UiElement;

    use crate::australia::rules::{cards::AustralianRegion, meta::GameMetaData};

    use super::*;

    #[test]
    /// Req 10. a
    fn test_10_a() {
        let mut player = AustraliaPlayer::new(0);
        player.hand.push(AustraliaCard::BarossaValley);
        player.discard_pile.push(AustraliaCard::BarossaValley);
        let scoring = Scoring::new().score_throw_catch(&player);
        assert_eq!(scoring.throw_catch(), 0);
        let mut player = AustraliaPlayer::new(0);
        player.hand.push(AustraliaCard::TheBungleBungles);
        player.discard_pile.push(AustraliaCard::Uluru);
        let scoring = Scoring::new().score_throw_catch(&player);
        assert_eq!(scoring.throw_catch(), 3);
    }

    #[test]
    /// Req 10. b
    fn test_10_b() {
        let mut player = AustraliaPlayer::new(0);
        player.hand.push(AustraliaCard::TheBungleBungles);
        player.hand.push(AustraliaCard::ThePinnacles);
        player.hand.push(AustraliaCard::MargaretRiver);
        player.hand.push(AustraliaCard::KalbarriNationalPark);

        let unclaimed_regions = vec![AustralianRegion::WesternAustralia] /* Initialize unclaimed regions */;
        let scoring = Scoring::new().score_regions(&mut player, &unclaimed_regions);
        assert_eq!(scoring.tourist_sites(), 7);
        assert_eq!(scoring.completed_regions().len(), 1);

        let mut player = AustraliaPlayer::new(0);
        player.hand.push(AustraliaCard::TheBungleBungles);
        player.hand.push(AustraliaCard::ThePinnacles);
        player.hand.push(AustraliaCard::MargaretRiver);
        player.hand.push(AustraliaCard::KalbarriNationalPark);

        let unclaimed_regions = vec![];
        let scoring = Scoring::new().score_regions(&mut player, &unclaimed_regions);
        assert_eq!(scoring.tourist_sites(), 4);
        assert_eq!(scoring.completed_regions().len(), 0);

        let mut player = AustraliaPlayer::new(0);
        player.hand.push(AustraliaCard::TheBungleBungles);
        player.hand.push(AustraliaCard::ThePinnacles);
        player.hand.push(AustraliaCard::MargaretRiver);
        player.hand.push(AustraliaCard::KalbarriNationalPark);

        let unclaimed_regions = vec![AustralianRegion::WesternAustralia];
        let scoring = Scoring::new().score_regions(&mut player, &unclaimed_regions);
        assert_eq!(scoring.tourist_sites(), 7);
        assert_eq!(scoring.completed_regions().len(), 1);

        let mut meta = GameMetaData::new(&[0, 1]);
        for player in meta.get_players().iter_mut() {
            player.hand.push(AustraliaCard::TheBungleBungles);
            player.hand.push(AustraliaCard::ThePinnacles);
            player.hand.push(AustraliaCard::MargaretRiver);
            player.hand.push(AustraliaCard::KalbarriNationalPark);
            player.hand.push(AustraliaCard::Uluru);
            player.hand.push(AustraliaCard::KakaduNationalPark);
            player.hand.push(AustraliaCard::NitmilukNationalPark);
            player.discard(&0).unwrap();
            player.show(&0).unwrap();
            player.show(&0).unwrap();
            player.show(&0).unwrap();
        }
        meta.score_round(&Vec::new());
        println!("{:?}", meta);
        for player in meta.get_players().iter() {
            println!("{:?}", player);
            // Expect 10 since 3 + number of elements in hand = 10
            assert_eq!(player.scoring[0].tourist_sites(), 10);
        }
        meta.new_round();
        for player in meta.get_players().iter_mut() {
            player.hand.push(AustraliaCard::TheBungleBungles);
            player.hand.push(AustraliaCard::ThePinnacles);
            player.hand.push(AustraliaCard::MargaretRiver);
            player.hand.push(AustraliaCard::KalbarriNationalPark);
            player.hand.push(AustraliaCard::Uluru);
            player.hand.push(AustraliaCard::KakaduNationalPark);
            player.hand.push(AustraliaCard::NitmilukNationalPark);
            player.discard(&0).unwrap();
            player.show(&0).unwrap();
            player.show(&0).unwrap();
            player.show(&0).unwrap();
        }
        meta.score_round(&Vec::new());
        println!("{:?}", meta);
        for player in meta.get_players() {
            println!("{:?}", player);
            // Expect 0 since 3 + number of elements in hand = 10
            assert_eq!(player.scoring[1].tourist_sites(), 0);
        }

        let mut meta = GameMetaData::new(&[0, 1]);
        for player in meta.get_players().iter_mut() {
            player.hand.push(AustraliaCard::TheBungleBungles);
            player.hand.push(AustraliaCard::ThePinnacles);
            player.hand.push(AustraliaCard::MargaretRiver);
            player.hand.push(AustraliaCard::Uluru);
            player.hand.push(AustraliaCard::KakaduNationalPark);
            player.hand.push(AustraliaCard::NitmilukNationalPark);
            player.discard(&0).unwrap();
            player.show(&0).unwrap();
            player.show(&0).unwrap();
            player.show(&0).unwrap();
        }
        meta.score_round(&Vec::new());
        println!("{:?}", meta);
        for player in meta.get_players().iter() {
            println!("{:?}", player);
            // Expect 10 since 3 + number of elements in hand = 10
            assert_eq!(player.scoring[0].tourist_sites(), 6);
        }
        meta.new_round();
        for player in meta.get_players().iter_mut() {
            player.hand.push(AustraliaCard::TheBungleBungles);
            player.hand.push(AustraliaCard::ThePinnacles);
            player.hand.push(AustraliaCard::MargaretRiver);
            player.hand.push(AustraliaCard::KalbarriNationalPark);
            player.hand.push(AustraliaCard::Uluru);
            player.hand.push(AustraliaCard::KakaduNationalPark);
            player.hand.push(AustraliaCard::NitmilukNationalPark);
            player.discard(&0).unwrap();
            player.show(&0).unwrap();
            player.show(&0).unwrap();
            player.show(&0).unwrap();
        }
        meta.score_round(&Vec::new());
        println!("{:?}", meta);
        for player in meta.get_players() {
            println!("{:?}", player);
            // Expect 0 since 3 + number of elements in hand = 10
            assert_eq!(player.scoring[1].tourist_sites(), 4);
        }
    }

    #[test]
    /// Req 10. c
    fn test_10_c() {
        let mut player = AustraliaPlayer::new(0);

        player.hand.push(AustraliaCard::TheBungleBungles); // 1 pt
        player.hand.push(AustraliaCard::KalbarriNationalPark); // 2 pt
        player.hand.push(AustraliaCard::MargaretRiver); // 3 pt
        player.hand.push(AustraliaCard::DaintreeRainforest); // 5 pt

        let scoring = Scoring::new().score_collections(&player);

        assert_eq!(scoring.collections(), 1 + 2 + 3 + 5);

        let mut player = AustraliaPlayer::new(0);

        player.hand.push(AustraliaCard::TheBungleBungles); // 1 pt
        player.hand.push(AustraliaCard::KalbarriNationalPark); // 2 pt
        player.hand.push(AustraliaCard::MargaretRiver); // 3 pt

        let scoring = Scoring::new().score_collections(&player);

        assert_eq!(scoring.collections(), (1 + 2 + 3) * 2);
    }
    #[test]
    fn test_10_d() {
        let mut meta = GameMetaData::new(&[0, 1]);

        for player in meta.get_players().iter_mut() {
            player.hand.push(AustraliaCard::ThePinnacles);
            player.hand.push(AustraliaCard::MargaretRiver);
            player.hand.push(AustraliaCard::Uluru);
            player.hand.push(AustraliaCard::LakeEyre);

            player.discard(&0).unwrap();
            player.show(&0).unwrap();
            player.show(&0).unwrap();
        }

        meta.score_round(&Vec::new());

        for player in meta.get_players().iter() {
            assert_eq!(player.scoring[0].animals(), 3 + 4);
        }
    }
    #[test]
    fn test_10_e() {
        let mut meta = GameMetaData::new(&[0, 1]);

        for player in meta.get_players().iter_mut() {
            player.hand.push(AustraliaCard::TheBungleBungles);
            player.hand.push(AustraliaCard::Uluru);
            player.hand.push(AustraliaCard::Uluru);
            player.hand.push(AustraliaCard::TheWhitsundays);
            player.hand.push(AustraliaCard::BlueMountains);
            player.hand.push(AustraliaCard::KingsCanyon);

            player.discard(&0).unwrap();
            player.show(&0).unwrap();
            player.show(&0).unwrap();
        }

        let activities_to_score = vec![
            (0, Some(AustralianActivity::IndigenousCulture)),
            (1, Some(AustralianActivity::IndigenousCulture)),
        ];

        meta.score_round(&activities_to_score);

        for player in meta.get_players().iter() {
            println!("{:?}", player);
            assert_eq!(player.scoring[0].activity(), 10);
        }
    }
    #[test]
    fn test_12() {
        let mut meta = GameMetaData::new(&[0, 1, 2, 3]);
        let scores = vec![
            Scoring::from_values(10, 21, 11, 1, 2, Vec::new()),
            Scoring::from_values(11, 20, 11, 1, 2, Vec::new()),
            Scoring::from_values(8, 20, 11, 1, 2, Vec::new()),
            Scoring::from_values(9, 20, 11, 1, 2, Vec::new()),
        ];
        // Simulate four rounds of scoring
        for _ in 0..4 {
            for (player, score) in meta.get_players().iter_mut().zip(scores.clone()) {
                player.add_score(score.clone());
                player.add_score(score.clone());
                player.add_score(score.clone());
                player.add_score(score.clone());
            }
        }

        let ranking = meta.rank();
        for ((idx, _), target) in ranking.iter().zip(vec![1, 0, 3, 2]) {
            assert_eq!(*idx, target);
        }
    }
}
