use serde::{Deserialize, Serialize};

use super::{
    cards::{Animal, AustralianActivity, AustralianAnimal, AustralianRegion, Card, Collection},
    AustraliaPlayer,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Scoring {
    throw_catch: usize,
    tourist_sites: usize,
    collections: usize,
    animals: usize,
    activity: usize,
    completed_regions: Vec<AustralianRegion>,
}
impl std::ops::AddAssign<Scoring> for Scoring {
    fn add_assign(&mut self, rhs: Scoring) {
        self.throw_catch += rhs.throw_catch;
        self.tourist_sites += rhs.tourist_sites;
        self.collections += rhs.collections;
        self.activity += rhs.activity;
        self.activity += rhs.activity;
        for region in rhs.completed_regions {
            if !self.completed_regions().contains(&region) {
                self.completed_regions.push(region);
            }
        }
    }
}
impl std::iter::Sum for Scoring {
    fn sum<I: Iterator<Item = Scoring>>(iter: I) -> Self {
        let mut total = Scoring::new();
        for el in iter {
            total += el;
        }
        total
    }
}
impl Scoring {
    pub fn new() -> Self {
        Self {
            throw_catch: 0,
            tourist_sites: 0,
            collections: 0,
            animals: 0,
            activity: 0,
            completed_regions: Vec::new(),
        }
    }
    pub fn from_values(
        throw_catch: usize,
        tourist_sites: usize,
        collections: usize,
        animals: usize,
        activity: usize,
        completed_regions: Vec<AustralianRegion>,
    ) -> Self {
        Self {
            throw_catch,
            tourist_sites,
            collections,
            animals,
            activity,
            completed_regions,
        }
    }
}
// Getters
impl Scoring {
    pub fn throw_catch(&self) -> usize {
        self.throw_catch
    }
    pub fn tourist_sites(&self) -> usize {
        self.tourist_sites
    }
    pub fn collections(&self) -> usize {
        self.collections
    }
    pub fn animals(&self) -> usize {
        self.animals
    }
    pub fn activity(&self) -> usize {
        self.activity
    }
    pub fn total_score(&self) -> usize {
        self.activity + self.animals + self.collections + self.throw_catch + self.tourist_sites
    }
    pub fn completed_regions(&self) -> Vec<AustralianRegion> {
        self.completed_regions.clone()
    }
}

// Builder pattern for scoring
impl Scoring {
    pub fn score_throw_catch(mut self, player: &AustraliaPlayer) -> Self {
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
    pub fn score_collections(mut self, player: &AustraliaPlayer) -> Self {
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

    pub fn score_animals(mut self, player: &AustraliaPlayer) -> Self {
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
    pub fn score_regions(
        mut self,
        player: &mut AustraliaPlayer,
        unclaimed_region: &Vec<AustralianRegion>,
    ) -> Self {
        let mut cards = player.get_discard();
        cards.extend(player.get_show());
        cards.extend(player.get_hand());
        let mut visited = player.get_visited();
        let mut total = 0;
        for card in cards {
            // Insert if not exists
            if !visited.contains(&card.to_char()) {
                visited.push(card.to_char());
                player.visit(card.to_char());
                //  A new site has been visited
                total += 1;
            }
        }
        let mut completed = Vec::new();
        for region in unclaimed_region {
            if region.completed(&visited) {
                completed.push(region.clone());
            }
        }
        total += completed.len() * 3;
        self.completed_regions.extend(completed);
        self.tourist_sites += total;
        self
    }
    pub fn score_activity(
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
        self.activity += match total {
            0 | 1 => 0,
            2 => 2,
            3 => 4,
            4 => 7,
            5 => 10,
            6 => 15,
            _ => unreachable!(),
        };
        self
    }
}
