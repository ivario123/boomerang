use std::fmt::Debug;

use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum CardError {
    NoSuchCard,
}
pub trait Card: Debug + Clone + Copy + Serialize + for<'a> Deserialize<'a> {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AustraliaCards {
    TheBungleBungles,
    ThePinnacles,
}

impl Card for AustraliaCards {
    /*
    fn as_u8<'a>(&'a self) -> &[u8] {
        use AustraliaCards::*;
        match self {
            TheBungleBungles => &[1],
            ThePinnacles => &[2],
        }
    }
    fn from_u8(stream: &[&u8]) -> Result<Self, CardError> {
        use AustraliaCards::*;
        match *stream[0] {
            1 => Ok(TheBungleBungles),
            2 => Ok(ThePinnacles),
            _ => Err(CardError::NoSuchCard),
        }
    }*/
}
