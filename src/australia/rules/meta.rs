//! Defines the game meta data type
//! 
//! This type hold all the data about the game that could be relevant at runtime.

use server::engine::rules::{Action, Error, New};
use tui::ui::UiElement;

use crate::australia::protocol::Event;

use super::{
    cards::{AustraliaDeck, AustralianActivity, AustralianRegion},
    scoring::Scoring,
    states::pass::Direction,
    AustraliaPlayer,
};

#[derive(Debug, Clone)]
pub struct GameMetaData {
    deck: AustraliaDeck,
    players: Vec<AustraliaPlayer>,
    non_completed_regions: Vec<AustralianRegion>,
    round_counter: usize,
}
// Getters
impl GameMetaData {
    pub fn get_players(&mut self) -> &mut Vec<AustraliaPlayer> {
        &mut self.players
    }
    pub fn rank(&mut self) -> Vec<(u8, Scoring)> {
        let mut totals = Vec::new();
        for player in &self.players {
            let mut sum = Scoring::new();
            for score in player.scores() {
                sum += score;
            }
            totals.push((player.id, sum));
        }
        totals.sort_by(|a, b| {
            let (a_tot, b_tot) = (a.1.total_score(), b.1.total_score());
            if a_tot > b_tot {
                std::cmp::Ordering::Less
            } else if a_tot == b_tot {
                match a.1.throw_catch() > b.1.throw_catch() {
                    true => std::cmp::Ordering::Less,
                    false => std::cmp::Ordering::Greater,
                }
            } else {
                std::cmp::Ordering::Greater
            }
        });
        println!("{:?}", totals);
        totals
    }
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
                .score_activity(player, activity)
                .score_animals(player);

            for el in scoring.completed_regions() {
                if !completed.contains(&el) {
                    completed.push(el)
                }
            }
            player.add_score(scoring);
        }
        for el in completed {
            let mut found = None;
            for (idx, region) in self.non_completed_regions.iter().enumerate() {
                if el == *region {
                    found = Some(idx);
                    break;
                }
            }
            if let Some(idx) = found {
                self.non_completed_regions.remove(idx);
            }
        }
        self.round_counter == 3
    }
    pub fn new_round(&mut self) {
        self.deck = AustraliaDeck::default();
        self.deck.shuffle();
        self.round_counter += 1;
        for player in self.players.iter_mut() {
            player.new_round();
        }
    }
}

impl GameMetaData {
    const MAX_CARDS: usize = 7;
    pub fn new(players: &[usize]) -> Self {
        let mut players_vec = Vec::with_capacity(players.len());
        for player in players {
            players_vec.push(AustraliaPlayer::new(*player as u8));
        }
        let mut deck = AustraliaDeck::default();
        deck.shuffle();
        Self {
            deck,
            players: players_vec,
            non_completed_regions: AustralianRegion::to_vec(),
            round_counter: 0,
        }
    }
    pub fn draft(&mut self) -> (bool, Vec<Action<New, Event>>) {
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
    pub fn discard(&mut self, uid: &usize, idx: &usize) -> Result<(), Error> {
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
    pub fn show(&mut self, uid: &usize, idx: &usize) -> Result<(), Error> {
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
    pub fn circulate(&mut self, direction: Direction) {
        let _players = match direction {
            Direction::Forward => {
                let mut prev_hand = self.players.last().unwrap().hand.clone();
                for player in self.players.iter_mut() {
                    let intermediate = player.hand.clone();
                    player.hand = prev_hand;
                    prev_hand = intermediate;
                }
            }
            Direction::Backward => {
                let mut prev_hand = self.players.first().unwrap().hand.clone();
                for player in self.players.iter_mut().rev() {
                    let intermediate = player.hand.clone();
                    player.hand = prev_hand;
                    prev_hand = intermediate;
                }
            }
        };
    }

    #[cfg(test)]
    pub fn hands(&mut self) -> Vec<AustraliaPlayer> {
        self.players.clone()
    }
    pub fn hands_singleton(&self) -> bool {
        let mut ret = true;
        for player in &self.players {
            if player.hand_size() != 1 {
                ret = false;
            }
        }
        ret
    }
}
