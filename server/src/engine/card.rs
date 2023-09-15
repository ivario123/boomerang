use std::fmt::Debug;

#[derive(Debug)]
pub enum CardError {
    NoSuchCard,
}

pub trait Card: Debug {
    fn as_u8<'a>(&'a self) -> &'a [u8];
    fn from_u8(stream: &[u8]) -> Result<Self, CardError>
    where
        Self: Sized;
}

#[derive(Debug)]
pub enum AustraliaCards {
    TheBungleBungles,
    ThePinnacles,
}

impl Card for AustraliaCards {
    fn as_u8<'a>(&'a self) -> &[u8] {
        use AustraliaCards::*;
        match self {
            TheBungleBungles => &[1],
            ThePinnacles => &[2],
        }
    }
    fn from_u8(stream: &[u8]) -> Result<Self, CardError> {
        use AustraliaCards::*;
        match stream[0] {
            1 => Ok(TheBungleBungles),
            2 => Ok(ThePinnacles),
            _ => Err(CardError::NoSuchCard),
        }
    }
}
