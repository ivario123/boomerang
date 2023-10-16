use rand::{thread_rng, seq::IteratorRandom};
use serde::{Serialize,Deserialize};

pub trait Card: std::fmt::Debug {
    fn to_char(&self) -> char;
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

impl AustraliaDeck {
    pub fn draft(&mut self) -> AustraliaCard {
        let idx = [0..self.deck.len()];
        let (drafted, _) = idx.iter().enumerate().choose(&mut thread_rng()).unwrap();
        self.deck.remove(drafted)
    }
}
macro_rules! card {
    ($(
        $name:ident : {
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
        site: 'A'
        region: "Western Australia"
        collection: "Leaves"
        activity: IndigenousCulture
    }

    ThePinnacles: {
        site: 'B'
        region: "Western Australia"
        animal: "Kangaroos"
        activity: Sightseeing
    }

    MargaretRiver: {
        site: 'C'
        region: "Western Australia"
        collection: "Shells"
        animal: "Kangaroos"
    }

    KalbarriNationalPark: {
        site: 'D'
        region: "Western Australia"
        collection: "Wildflowers"
        activity: Bushwalking
    }

    Uluru: {
        site: 'E'
        region: "Northern Territory"
        animal: "Emus"
        activity: IndigenousCulture
    }

    KakaduNationalPark: {
        site: 'F'
        region: "Northern Territory"
        animal: "Wombats"
        activity: Sightseeing
    }

    NitmilukNationalPark: {
        site: 'G'
        region: "Northern Territory"
        collection: "Shells"
        animal: "Platypuses"
    }

    KingsCanyon: {
        site: 'H'
        region: "Northern Territory"
        animal: "Koalas"
        activity: Swimming
    }

    TheGreatBarrierReef: {
        site: 'I'
        region: "Queensland"
        collection: "Wildflowers"
        activity: Sightseeing
    }

    TheWhitsundays: {
        site: 'J'
        region: "Queensland"
        animal: "Kangaroos"
        activity: IndigenousCulture
    }

    DaintreeRainforest: {
        site: 'K'
        region: "Queensland"
        collection: "Souvenirs"
        activity: Bushwalking
    }

    SurfersParadise: {
        site: 'L'
        region: "Queensland"
        collection: "Wildflowers"
        activity: Swimming
    }

    BarossaValley: {
        site: 'M'
        region: "South Australia"
        animal: "Koalas"
        activity: Bushwalking
    }

    LakeEyre: {
        site: 'N'
        region: "South Australia"
        animal: "Emus"
        activity: Swimming
    }

    KangarooIsland: {
        site: 'O'
        region: "South Australia"
        animal: "Kangaroos"
        activity: Bushwalking
    }

    MountGambier: {
        site: 'P'
        region: "South Australia"
        collection: "Wildflowers"
        activity: Sightseeing
    }

    BlueMountains: {
        site: 'Q'
        region: "New South Whales"
        activity: IndigenousCulture
    }

    SydneyHarbour: {
        site: 'R'
        region: "New South Whales"
        animal: "Emus"
        activity: Sightseeing
    }

    BondiBeach: {
        site: 'S'
        region: "New South Whales"
        activity: Swimming
    }

    HunterValley: {
        site: 'T'
        region: "New South Whales"
        animal: "Emus"
        activity: Bushwalking
    }

    Melbourne: {
        site: 'U'
        region: "Victoria"
        animal: "Wombats"
        activity: Bushwalking
    }

    TheMCG: {
        site: 'V'
        region: "Victoria"
        collection: "Leaves"
        activity: IndigenousCulture
    }

    TwelveApostles: {
        site: 'W'
        region: "Victoria"
        collection: "Shells"
        activity: Swimming
    }

    RoyalExhibitionBuilding: {
        site: 'X'
        region: "Victoria"
        collection: "Leaves"
    }

    SalamancaMarkets: {
        site: 'Y'
        region: "Tasmania"
        collection: "Leaves"
        animal: "Emus"
    }

    MountWellington: {
        site: 'Z'
        region: "Tasmania"
        animal: "Koalas"
        activity: Sightseeing
    }

    PortArthur: {
        site: '*'
        region: "Tasmania"
        collection: "Leaves"
        activity: IndigenousCulture
    }

    Richmond: {
        site: '-'
        region: "Tasmania"
        animal: "Kangaroos"
        activity: Swimming
    }
}