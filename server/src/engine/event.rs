use crate::engine::card::AustraliaCards;

use super::card::Card;

#[repr(u8)]
#[derive(Debug,Clone,Copy)]
pub enum Event {
    Deal(AustraliaCards),
    ListLobby,
    CreateLobby,
    JoinLobby(u8),
    KeepAlive,
    KeepAliveResponse,
}

/// Enumerates all of the possible errors for the [`Event`] enum
pub enum EventError {
    /// Thrown when the parsing of a bitstream fails.
    InvalidBitStream,
}

impl<'a> Into<Vec<u8>> for &'a Event {
    fn into(self) -> Vec<u8> {
        use Event::*;
        match self {
            Deal(card) => {
                let card_rerp: &'a [u8] = card.as_u8();
                let ret = [&[1], &card_rerp[..]].concat();
                ret
            }
            ListLobby => {
                vec![2]
            }
            CreateLobby => {
                vec![3]
            }
            JoinLobby(id) => Vec::from([4, *id]),
            KeepAlive => vec![5],
            KeepAliveResponse => vec![6],
        }
    }
}

impl TryInto<Event> for Vec<u8> {
    type Error = EventError;
    fn try_into(self) -> Result<Event, EventError> {
        use Event::*;
        // First element identifies type of event
        match self[0] {
            0 => Ok(KeepAlive),
            1 => match self.len() > 1 {
                false => Err(EventError::InvalidBitStream),
                _ => match AustraliaCards::from_u8(&self[1..]) {
                    Ok(card) => Ok(Deal(card)),
                    Err(_) => Err(EventError::InvalidBitStream),
                },
            },
            2 => Ok(ListLobby),
            3 => Ok(CreateLobby),
            4 => match self.len() > 1 {
                true => Ok(JoinLobby(self[1])),
                false => Err(EventError::InvalidBitStream),
            },
            5 => Ok(KeepAlive),
            6 => Ok(KeepAliveResponse),
            _ => Err(EventError::InvalidBitStream),
        }
    }
}
