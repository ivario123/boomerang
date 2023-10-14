use crate::engine::card::AustraliaCards;

use super::card::Card;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy)]
pub struct SendableCard<C: Card> {
    card: C,
}

pub trait GameEvent:
    Clone + Serialize + for<'a> Deserialize<'a> + PartialEq + From<BackendEvent> + Into<Vec<u8>> + std::fmt::Debug + Send + Sync
{
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Definiton of protocol events.
pub enum BackendEvent {
    Connected(u8),
    UnexpectedMessage,
}

impl GameEvent for BackendEvent {}

/// Enumerates all of the possible errors for the [`Event`] enum
#[derive(Debug)]
pub enum EventError {
    /// Thrown when the parsing of a bitstream fails.
    InvalidBitStream,
}

impl Into<Vec<u8>> for BackendEvent {
    fn into(self) -> Vec<u8> {
        serde_json::to_string(&self).unwrap().into_bytes()
    }
}

#[cfg(test)]
mod test {
    use super::BackendEvent as Event;
    #[test]
    pub fn test_serialize_distinct_type() {
        //
        let data = Event::Connected(0);
        let serialized = serde_json::to_string(&data).unwrap();
        let intermediate = serialized.as_bytes();
        let returned_string = String::from_utf8(intermediate.to_vec()).unwrap();
        let parsed = serde_json::from_str(&returned_string).unwrap();
        assert_eq!(data, parsed);
    }
    #[test]
    pub fn test_serialize_vec() {
        //
        let data = vec![Event::Connected(0)];
        let serialized = serde_json::to_string(&data).unwrap();
        let intermediate = serialized.as_bytes();
        let returned_string = String::from_utf8(intermediate.to_vec()).unwrap();
        let parsed = serde_json::from_str::<Vec<Event>>(&returned_string).unwrap();
        assert_eq!(data, parsed);
    }
}
