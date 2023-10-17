use rand::{seq::IteratorRandom, thread_rng};
use serde::{Deserialize, Serialize};

pub trait Card: std::fmt::Debug {
    fn to_char(&self) -> char;
    fn name(&self) -> &str;
}

macro_rules! activities {
    ($($variant:ident),*) => {
        #[derive(Debug, Serialize, Deserialize, Clone,PartialEq)]
        pub enum AustralianActivities {
            $($variant,)*
        }

        impl AustralianActivities {
            pub fn to_vec() -> Vec<AustralianActivities> {
                vec![$(AustralianActivities::$variant,)*]
            }
        }
    };
}

activities! {
    IndigenousCulture,
    Sightseeing,
    Bushwalking,
    Swimming,
    Souvenirs
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AustraliaDeck {
    deck: Vec<AustraliaCard>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AustraliaHand {
    deck: Vec<AustraliaCard>,
}

impl AustraliaDeck {
    pub fn draft(&mut self) -> AustraliaCard {
        let idx = [0..self.deck.len()];
        let (drafted, _) = idx.iter().enumerate().choose(&mut thread_rng()).unwrap();
        self.deck.remove(drafted)
    }
}

impl tui::ui::UiElement for AustraliaCard {
    /// This should never be called
    fn new() -> Self {
        panic!("Card is not to be instantiated like this")
    }
}
impl tui::ui::Card for AustraliaCard {
    fn get_name(&self) -> &str {
        self.name()
    }
}



macro_rules! card {
    ($(
        $name:ident : {
            name : $str_rpr:literal
            site : $site:literal
            region : $region:literal
            $(collection : $collection:literal)?
            $(animal : $animal:literal)?
            $(activity : $activity:ident)?
        }
    )+) => {
        #[derive(Debug,Serialize,Deserialize,Clone,Copy,PartialEq)]
        pub enum AustraliaCard{
            $(
                $name,
            )+
        }
        impl Card for AustraliaCard {
            fn to_char(&self) -> char{
                match self{
                    $(
                        AustraliaCard::$name => {$site},
                    )+
                }
            }

            fn name(&self) -> &str{
                match self{
                    $(
                        AustraliaCard::$name => $str_rpr,
                    )+
                }

            }
        }
        impl Default for AustraliaDeck{
            fn default() -> Self{
                Self{
                    deck:vec![$(AustraliaCard::$name,)+]
                }
            }
        }
    };
}
card! {
    TheBungleBungles: {
        name: "The Bungle Bungles"
        site: 'A'
        region: "Western Australia"
        collection: "Leaves"
        activity: IndigenousCulture
    }

    ThePinnacles: {
        name: "The Pinnacles"
        site: 'B'
        region: "Western Australia"
        animal: "Kangaroos"
        activity: Sightseeing
    }

    MargaretRiver: {
        name: "Margaret River"
        site: 'C'
        region: "Western Australia"
        collection: "Shells"
        animal: "Kangaroos"
    }

    KalbarriNationalPark: {
        name: "Kalbarri National Park"
        site: 'D'
        region: "Western Australia"
        collection: "Wildflowers"
        activity: Bushwalking
    }

    Uluru: {
        name: "Uluru"
        site: 'E'
        region: "Northern Territory"
        animal: "Emus"
        activity: IndigenousCulture
    }

    KakaduNationalPark: {
        name: "Kakadu National Park"
        site: 'F'
        region: "Northern Territory"
        animal: "Wombats"
        activity: Sightseeing
    }

    NitmilukNationalPark: {
        name: "Nitmiluk National Park"
        site: 'G'
        region: "Northern Territory"
        collection: "Shells"
        animal: "Platypuses"
    }

    KingsCanyon: {
        name: "King's Canyon"
        site: 'H'
        region: "Northern Territory"
        animal: "Koalas"
        activity: Swimming
    }

    TheGreatBarrierReef: {
        name: "The Great Barrier Reef"
        site: 'I'
        region: "Queensland"
        collection: "Wildflowers"
        activity: Sightseeing
    }

    TheWhitsundays: {
        name: "The Whitsundays"
        site: 'J'
        region: "Queensland"
        animal: "Kangaroos"
        activity: IndigenousCulture
    }

    DaintreeRainforest: {
        name: "Daintree Rainforest"
        site: 'K'
        region: "Queensland"
        collection: "Souvenirs"
        activity: Bushwalking
    }

    SurfersParadise: {
        name: "Surfers Paradise"
        site: 'L'
        region: "Queensland"
        collection: "Wildflowers"
        activity: Swimming
    }

    BarossaValley: {
        name: "Barossa Valley"
        site: 'M'
        region: "South Australia"
        animal: "Koalas"
        activity: Bushwalking
    }

    LakeEyre: {
        name: "Lake Eyre"
        site: 'N'
        region: "South Australia"
        animal: "Emus"
        activity: Swimming
    }

    KangarooIsland: {
        name: "Kangaroo Island"
        site: 'O'
        region: "South Australia"
        animal: "Kangaroos"
        activity: Bushwalking
    }

    MountGambier: {
        name: "Mount Gambier"
        site: 'P'
        region: "South Australia"
        collection: "Wildflowers"
        activity: Sightseeing
    }

    BlueMountains: {
        name: "Blue Mountains"
        site: 'Q'
        region: "New South Whales"
        activity: IndigenousCulture
    }

    SydneyHarbour: {
        name:  "Sydney Harbour"
        site: 'R'
        region: "New South Whales"
        animal: "Emus"
        activity: Sightseeing
    }

    BondiBeach: {
        name: "Bondi Beach"
        site: 'S'
        region: "New South Whales"
        activity: Swimming
    }

    HunterValley: {
        name: "Hunter Valley"
        site: 'T'
        region: "New South Whales"
        animal: "Emus"
        activity: Bushwalking
    }

    Melbourne: {
        name: "Melbourne"
        site: 'U'
        region: "Victoria"
        animal: "Wombats"
        activity: Bushwalking
    }

    TheMCG: {
        name: "The MCG"
        site: 'V'
        region: "Victoria"
        collection: "Leaves"
        activity: IndigenousCulture
    }

    TwelveApostles: {
        name: "Twelve Apostles"
        site: 'W'
        region: "Victoria"
        collection: "Shells"
        activity: Swimming
    }

    RoyalExhibitionBuilding: {
        name: "Royal Exhibition Building"
        site: 'X'
        region: "Victoria"
        collection: "Leaves"
    }

    SalamancaMarkets: {
        name: "Salamanca Markets"
        site: 'Y'
        region: "Tasmania"
        collection: "Leaves"
        animal: "Emus"
    }

    MountWellington: {
        name: "Mount Wellington"
        site: 'Z'
        region: "Tasmania"
        animal: "Koalas"
        activity: Sightseeing
    }

    PortArthur: {
        name: "Port Arthur"
        site: '*'
        region: "Tasmania"
        collection: "Leaves"
        activity: IndigenousCulture
    }

    Richmond: {
        name: "Richmond"
        site: '-'
        region: "Tasmania"
        animal: "Kangaroos"
        activity: Swimming
    }
}
