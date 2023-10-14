use crate::engine::card::AustraliaCards;

use super::card::Card;

#[derive(Debug, Clone, Copy)]
/// Definiton of protocol events.
pub enum Event {
    Deal(AustraliaCards),
    ListLobby,
    CreateLobby,
    JoinLobby(u8),
    KeepAlive,
    KeepAliveResponse,
    Connected(u8),
    UnexpectedMessage,
}

/// Enumerates all of the possible errors for the [`Event`] enum
#[derive(Debug)]
pub enum EventError {
    /// Thrown when the parsing of a bitstream fails.
    InvalidBitStream,
}

impl<'a> Into<Vec<u8>> for &'a Event {
    fn into(self) -> Vec<u8> {
        use Event::*;
        let mut ret = match self {
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
            Connected(uid) => {
                vec![7, *uid]
            }
            UnexpectedMessage => {
                vec![8]
            }
        };
        ret.push(0);
        return ret;
    }
}

impl TryInto<Event> for Vec<&u8> {
    type Error = EventError;
    fn try_into(self) -> Result<Event, EventError> {
        use Event::*;
        // First element identifies type of event
        match *self[0] {
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
                true => Ok(JoinLobby(*self[1])),
                false => Err(EventError::InvalidBitStream),
            },
            5 => Ok(KeepAlive),
            6 => Ok(KeepAliveResponse),
            7 => match self.len() > 1 {
                true => Ok(Connected(*self[1])),
                false => Err(EventError::InvalidBitStream),
            },
            8 => Ok(UnexpectedMessage),
            _ => Err(EventError::InvalidBitStream),
        }
    }
}

pub struct EventList(pub Vec<Result<Event, EventError>>);

impl Into<EventList> for Vec<u8> {
    fn into(self) -> EventList {
        // First find all 0 indecies
        let mut indecies: Vec<(usize, usize)> = Vec::new();
        let mut prev_idx = 0;
        for (idx, el) in self.iter().enumerate() {
            match self.get(idx + 1) {
                Some(next) => {
                    if *el != 0 && *next == 0 {
                        indecies.push((prev_idx, idx + 1));
                        prev_idx = idx + 2;
                    }
                }
                _ => {}
            }
        }
        let mut events = Vec::new();
        for (start, stop) in indecies {
            let msg: Vec<&u8> = self[start..stop].iter().collect();
            events.push(msg.try_into());
            // First element identifies type of event
        }
        EventList(events)
    }
}
