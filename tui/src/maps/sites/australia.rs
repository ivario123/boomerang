#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Region {
    WesternAustralia,
    NorthernTerritory,
    Queensland,
    SouthAustralia,
    NewSouthWhales,
    Victoria,
    Tasmania,
}

impl super::Region for Region {
    fn default() -> Self {
        Region::WesternAustralia
    }
    fn coordinates(&self) -> (f64, f64) {
        match self {
            Region::WesternAustralia => (82.0, 220.0),
            Region::NorthernTerritory => (220.0, 300.0),
            Region::Queensland => (340.0, 280.0),
            Region::SouthAustralia => (230.0, 180.0),
            Region::NewSouthWhales => (360.0, 150.0),
            Region::Victoria => (340.0, 87.5),
            Region::Tasmania => (355.0, 50.0),
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
		impl Region{
			pub fn sites() -> Vec<super::TouristSite<Region>>{
				[
					$(
						$(
							super::TouristSite::new($site.to_owned(), $id.to_owned() ,Region::$region),
						)+
					)+
				].to_vec()
			}
		}
	};
}

tourist_sites!(
    WesternAustralia : {
        ("The Bungle Bungles","A")
        ("The Pinnacles","B")
        ("Margaret River","C")
        ("Kalbarri National Park","D")
    }
    NorthernTerritory : {
        ("Uluru","E")
        ("Kakadu National Park","F")
        ("Nitmiluk National Park","G")
        ("King's Canyon","H")
    }
    Queensland : {
        ("The Great Barrier Reef","I")
        ("The Whitsundays","J")
        ("Daintree Rainforest","K")
        ("Surfers Paradise","L")
    }
    SouthAustralia : {
        ("Barossa Valley","M")
        ("Lake Eyre","N")
        ("Kangaroo Island","O")
        ("Mount Gambier","P")
    }
    NewSouthWhales : {
        ("Blue Mountains","Q")
        ("Sydney Harbour","R")
        ("Bondi Beach","S")
        ("Hunter Valley","T")
    }
    Victoria : {
        ("Melbourne","U")
        ("The MCG","V")
        ("Twelve Apostles","W")
        ("Royal Exhibition Building","X")
    }
    Tasmania : {
        ("Salamanca Markets","Y")
        ("Mount Wellington","Z")
        ("Port Arthur","*")
        ("Richmond","-")
    }
);
