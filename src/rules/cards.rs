use log::error;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

pub trait Collection {
    fn score(&self) -> usize;
}
pub trait Animal: PartialEq {
    fn score(&self) -> usize;
}

pub trait Card<C: Collection, A: Animal>: std::fmt::Debug {
    fn to_char(&self) -> char;
    fn name(&self) -> &str;
    fn number(&self) -> usize;
    fn collection(&self) -> Option<C>;
    fn animal(&self) -> Option<A>;
    fn activity(&self) -> Option<AustralianActivity>;
    fn region(&self) -> AustralianRegion;
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
    /// Pops the last card from the deck, given that the deck is shuffled this is random.
    pub fn draft(&mut self) -> AustraliaCard {
        match self.deck.pop() {
            Some(card) => card,
            _ => {
                error!("Tried to draft a card from an empty hand");
                panic!()
            }
        }
    }
    #[cfg(test)]
    pub fn cards(&mut self) -> Vec<AustraliaCard> {
        self.deck.clone()
    }
    pub fn shuffle(&mut self) {
        self.deck.shuffle(&mut rand::thread_rng());
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
    fn number(&self) -> usize {
        Card::number(self)
    }
}

macro_rules! regions {
    ($(($variant:ident,$name:literal, $number:literal $($site:literal)+))*) => {
        #[derive(Debug, Serialize, Deserialize, Clone,Copy,PartialEq)]
        pub enum AustralianRegion {
            $($variant,)*
        }
        #[allow(dead_code)]
        impl AustralianRegion {
            pub fn to_vec() -> Vec<AustralianRegion> {
                vec![$(AustralianRegion::$variant,)*]
            }
            pub fn to_string_vec(cards : Vec<AustralianRegion>) -> Vec<String>{
                let mut ret = Vec::new();
                for card in cards{
                    ret.push(
                        match card{
                            $(
                                AustralianRegion::$variant => $name.to_owned(),
                            )+
                        }
                    )
                }
                ret
            }
            pub fn number(&self) -> usize{
                match self {
                    $(
                        AustralianRegion::$variant => $number,
                    )+
                }
            }
            pub fn completed(self,identifiers:&Vec<char>) -> bool{
                match self{
                    $(
                        AustralianRegion::$variant => {
                            let mut completed = true;
                            for el in vec![$($site,)+]{
                                if !identifiers.contains(&el){
                                   completed = false;
                                }
                            }
                            completed
                        }
                    )+
                }
            }
        }
    };
}
macro_rules! activities {
    ($(($variant:ident,$name:literal)),*) => {
        #[derive(Debug, Serialize, Deserialize, Clone,Copy,PartialEq)]
        pub enum AustralianActivity {
            $($variant,)*
        }

        impl Into<String> for AustralianActivity{
            fn into(self) -> String{
                match self{
                    $(
                        AustralianActivity::$variant => $name.to_owned(),
                    )+
                }
            }
        }
        impl AustralianActivity {
            pub fn to_vec() -> Vec<AustralianActivity> {
                vec![$(AustralianActivity::$variant,)*]
            }
            pub fn to_string_vec(cards : Vec<AustralianActivity>) -> Vec<String>{
                let mut ret = Vec::new();
                for card in cards{
                    ret.push(card.into())
                }
                ret
            }
        }
    };
}
macro_rules! collections {
    ($(($variant:ident,$name:literal,$score:literal))+) => {
        #[derive(Debug, Serialize, Deserialize, Clone,Copy,PartialEq)]
        pub enum AustralianCollection {
            $($variant,)*
        }

        impl Into<String> for AustralianCollection{
            fn into(self) -> String{
                match self{
                    $(
                        AustralianCollection::$variant => $name.to_owned(),
                    )+
                }
            }
        }
        #[allow(dead_code)]
        impl AustralianCollection {
            pub fn to_vec() -> Vec<AustralianCollection> {
                vec![$(AustralianCollection::$variant,)*]
            }
            pub fn to_string_vec(collections : Vec<AustralianCollection>) -> Vec<String>{
                let mut ret = Vec::new();
                for collection in collections{
                    ret.push(
                        collection.into()
                    )
                }
                ret
            }
        }
        impl Collection for AustralianCollection{
            fn score(&self) -> usize{

                match self{
                    $(
                        AustralianCollection::$variant => $score,
                    )+
                }
            }
        }
    };
}
macro_rules! animals {
    ($(($variant:ident,$name:literal,$score:literal))+) => {
        #[derive(Debug, Serialize, Deserialize, Clone,Copy,PartialEq,Hash,Eq)]
        pub enum AustralianAnimal {
            $($variant,)*
        }

        impl Into<String> for AustralianAnimal{
            fn into(self) -> String{
                match self{
                    $(
                        AustralianAnimal::$variant => $name.to_owned(),
                    )+
                }
            }
        }
        #[allow(dead_code)]
        impl AustralianAnimal {
            pub fn to_vec() -> Vec<AustralianAnimal> {
                vec![$(AustralianAnimal::$variant,)*]
            }
            pub fn to_string_vec(collections : Vec<AustralianAnimal>) -> Vec<String>{
                let mut ret = Vec::new();
                for collection in collections{
                    ret.push(
                        collection.into()
                    )
                }
                ret
            }
        }
        impl Animal for AustralianAnimal{
            fn score(&self) -> usize{
                match self{
                    $(
                        AustralianAnimal::$variant => $score,
                    )+
                }
            }
        }
    };
}
animals! {
    (Kangaroos,"Kangaroos",3)
    (Emus,"Emus",4)
    (Wombats,"Wombats",5)
    (Koalas,"Koalas",7)
    (Platypuses,"Platypuses",9)
}

collections! {
    (Leaves,"Leaves",1)
    (Wildflowers,"Wildflowers",2)
    (Shells,"Shells",3)
    (Souvenirs,"Souvenirs",5)
}

activities! {
    (IndigenousCulture,"Indigenous culture"),
    (Sightseeing,"Sightseeing"),
    (Bushwalking,"Bushwalking"),
    (Swimming,"Swimming"),
    (Souvenirs,"Souvenirs")
}

regions! {
    (WesternAustralia,"Western Australia", 1 'A' 'B' 'C' 'D')
    (NorthernTerritory,"Northern Territory", 4 'E' 'F' 'G' 'H')
    (Queensland,"Queensland",6 'I' 'J' 'K' 'L')
    (SouthAustralia ,"South Australia", 3 'M' 'N' 'O' 'P')
    (NewSouthWhales ,"New South Whales", 5 'Q' 'R' 'S' 'T')
    (Victoria ,"Victoria", 2 'U' 'V' 'W' 'X')
    (Tasmania ,"Tasmania ", 7 'Y' 'Z' '*' '-')
}

macro_rules! card {
    ($(
        $name:ident : {
            name : $str_rpr:literal
            site : $site:literal
            region : $region:ident
            $(collection : $collection:ident)?
            $(animal : $animal:ident)?
            $(activity : $activity:ident)?
        }
    )+) => {
        #[derive(Debug,Serialize,Deserialize,Clone,Copy,PartialEq)]
        pub enum AustraliaCard{
            $(
                $name,
            )+
        }

        impl Card<AustralianCollection,AustralianAnimal> for AustraliaCard {
            fn to_char(&self) -> char{
                match self{
                    $(
                        AustraliaCard::$name => {$site},
                    )+
                }
            }

            fn region(&self) -> AustralianRegion{
                match self{
                    $(
                        AustraliaCard::$name => AustralianRegion::$region,
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

            fn number(&self) -> usize{
                match self{
                    $(
                        AustraliaCard::$name => AustralianRegion::$region.number(),
                    )+
                }
            }
            fn collection(&self) -> Option<AustralianCollection>{

                match self{
                    $(
                        AustraliaCard::$name => {
                            $(return Some(AustralianCollection::$collection);)?

                            #[allow(unreachable_code)]
                            None
                        },
                    )+
                }
            }
            fn animal(&self) -> Option<AustralianAnimal>{

                match self{
                    $(
                        AustraliaCard::$name => {
                            $(return Some(AustralianAnimal::$animal);)?
                            #[allow(unreachable_code)]
                            None
                        },
                    )+
                }
            }
            fn activity(&self) -> Option<AustralianActivity>{

                match self{
                    $(
                        AustraliaCard::$name => {
                            $(return Some(AustralianActivity::$activity);)?
                            #[allow(unreachable_code)]
                            None
                        },
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
        region: WesternAustralia
        collection: Leaves
        activity: IndigenousCulture
    }

    ThePinnacles: {
        name: "The Pinnacles"
        site: 'B'
        region: WesternAustralia
        animal: Kangaroos
        activity: Sightseeing
    }

    MargaretRiver: {
        name: "Margaret River"
        site: 'C'
        region: WesternAustralia
        collection: Shells
        animal: Kangaroos
    }

    KalbarriNationalPark: {
        name: "Kalbarri National Park"
        site: 'D'
        region: WesternAustralia
        collection: Wildflowers
        activity: Bushwalking
    }

    Uluru: {
        name: "Uluru"
        site: 'E'
        region: NorthernTerritory
        animal: Emus
        activity: IndigenousCulture
    }

    KakaduNationalPark: {
        name: "Kakadu National Park"
        site: 'F'
        region: NorthernTerritory
        animal: Wombats
        activity: Sightseeing
    }

    NitmilukNationalPark: {
        name: "Nitmiluk National Park"
        site: 'G'
        region: NorthernTerritory
        collection: Shells
        animal: Platypuses
    }

    KingsCanyon: {
        name: "King's Canyon"
        site: 'H'
        region: NorthernTerritory
        animal: Koalas
        activity: Swimming
    }

    TheGreatBarrierReef: {
        name: "The Great Barrier Reef"
        site: 'I'
        region: Queensland
        collection: Wildflowers
        activity: Sightseeing
    }

    TheWhitsundays: {
        name: "The Whitsundays"
        site: 'J'
        region: Queensland
        animal: Kangaroos
        activity: IndigenousCulture
    }

    DaintreeRainforest: {
        name: "Daintree Rainforest"
        site: 'K'
        region: Queensland
        collection: Souvenirs
        activity: Bushwalking
    }

    SurfersParadise: {
        name: "Surfers Paradise"
        site: 'L'
        region: Queensland
        collection: Wildflowers
        activity: Swimming
    }

    BarossaValley: {
        name: "Barossa Valley"
        site: 'M'
        region: SouthAustralia
        animal: Koalas
        activity: Bushwalking
    }

    LakeEyre: {
        name: "Lake Eyre"
        site: 'N'
        region: SouthAustralia
        animal: Emus
        activity: Swimming
    }

    KangarooIsland: {
        name: "Kangaroo Island"
        site: 'O'
        region: SouthAustralia
        animal: Kangaroos
        activity: Bushwalking
    }

    MountGambier: {
        name: "Mount Gambier"
        site: 'P'
        region: SouthAustralia
        collection: Wildflowers
        activity: Sightseeing
    }

    BlueMountains: {
        name: "Blue Mountains"
        site: 'Q'
        region: NewSouthWhales
        activity: IndigenousCulture
    }

    SydneyHarbour: {
        name:  "Sydney Harbour"
        site: 'R'
        region: NewSouthWhales
        animal: Emus
        activity: Sightseeing
    }

    BondiBeach: {
        name: "Bondi Beach"
        site: 'S'
        region: NewSouthWhales
        activity: Swimming
    }

    HunterValley: {
        name: "Hunter Valley"
        site: 'T'
        region: NewSouthWhales
        animal: Emus
        activity: Bushwalking
    }

    Melbourne: {
        name: "Melbourne"
        site: 'U'
        region: Victoria
        animal: Wombats
        activity: Bushwalking
    }

    TheMCG: {
        name: "The MCG"
        site: 'V'
        region: Victoria
        collection: Leaves
        activity: IndigenousCulture
    }

    TwelveApostles: {
        name: "Twelve Apostles"
        site: 'W'
        region: Victoria
        collection: Shells
        activity: Swimming
    }

    RoyalExhibitionBuilding: {
        name: "Royal Exhibition Building"
        site: 'X'
        region: Victoria
        collection: Leaves
    }

    SalamancaMarkets: {
        name: "Salamanca Markets"
        site: 'Y'
        region: Tasmania
        collection: Leaves
        animal: Emus
    }

    MountWellington: {
        name: "Mount Wellington"
        site: 'Z'
        region: Tasmania
        animal: Koalas
        activity: Sightseeing
    }

    PortArthur: {
        name: "Port Arthur"
        site: '*'
        region: Tasmania
        collection: Leaves
        activity: IndigenousCulture
    }

    Richmond: {
        name: "Richmond"
        site: '-'
        region: Tasmania
        animal: Kangaroos
        activity: Swimming
    }
}
