//! Maps out all of the australian regions 

use tui::maps::sites::{Region as RegionTrait, TouristSite};

use crate::australia::rules::cards::AustralianRegion;

impl RegionTrait for AustralianRegion {
    fn default() -> Self {
        AustralianRegion::WesternAustralia
    }
    fn coordinates(&self) -> (f64, f64) {
        match self {
            AustralianRegion::WesternAustralia => (82.0, 220.0),
            AustralianRegion::NorthernTerritory => (220.0, 300.0),
            AustralianRegion::Queensland => (340.0, 280.0),
            AustralianRegion::SouthAustralia => (230.0, 180.0),
            AustralianRegion::NewSouthWhales => (360.0, 150.0),
            AustralianRegion::Victoria => (340.0, 87.5),
            AustralianRegion::Tasmania => (355.0, 50.0),
        }
    }
}


macro_rules! tourist_sites {
	($(
		$region:ident : {
			$(
				($site:literal,$id:literal)
			)+
		}
	)+) => {
		impl AustralianRegion{
			pub fn sites() -> Vec<TouristSite<AustralianRegion>>{
				[
					$(
						$(
							TouristSite::new($site.to_owned(), $id ,AustralianRegion::$region),
						)+
					)+
				].to_vec()
			}
		}
	};
}

tourist_sites!(
    WesternAustralia : {
        ("The Bungle Bungles",'A')
        ("The Pinnacles",'B')
        ("Margaret River",'C')
        ("Kalbarri National Park",'D')
    }
    NorthernTerritory : {
        ("Uluru",'E')
        ("Kakadu National Park",'F')
        ("Nitmiluk National Park",'G')
        ("King's Canyon",'H')
    }
    Queensland : {
        ("The Great Barrier Reef",'I')
        ("The Whitsundays",'J')
        ("Daintree Rainforest",'K')
        ("Surfers Paradise",'L')
    }
    SouthAustralia : {
        ("Barossa Valley",'M')
        ("Lake Eyre",'N')
        ("Kangaroo Island",'O')
        ("Mount Gambier",'P')
    }
    NewSouthWhales : {
        ("Blue Mountains",'Q')
        ("Sydney Harbour",'R')
        ("Bondi Beach",'S')
        ("Hunter Valley",'T')
    }
    Victoria : {
        ("Melbourne",'U')
        ("The MCG",'V')
        ("Twelve Apostles",'W')
        ("Royal Exhibition Building",'X')
    }
    Tasmania : {
        ("Salamanca Markets",'Y')
        ("Mount Wellington",'Z')
        ("Port Arthur",'*')
        ("Richmond",'-')
    }
);
