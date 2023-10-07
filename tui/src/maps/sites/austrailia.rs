#[derive(Clone, Copy, Debug)]
pub enum Region {
    WesternAustralia,
    NorthernTerritory,
    Queensland,
    SouthAustralia,
    NewSouthWhales,
    Victoria,
    Tasmania,
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
			fn sites() -> Vec<super::TouristSite<Region>>{
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

impl super::Region for Region {
    fn coordinates() -> (f64, f64) {
        todo!()
    }
}
