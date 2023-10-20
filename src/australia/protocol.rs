use serde::{Deserialize, Serialize};
use server::engine::event::{BackendEvent, GameEvent};
use tui::ui::UiMessage;

use super::rules::{
    cards::{AustraliaCard, AustralianActivity},
    scoring::Scoring,
    AustraliaPlayer,
};

/// Events sent to and from the [`server`].
///
/// Not all of these events require a response from the player.
/// To check wether an event requires a response use [`GameEvent::requires_response`]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Event {
    ReadyCheck,
    Accept,
    Deny,
    Deal(AustraliaCard),
    Hand(Vec<AustraliaCard>),
    ShowRequest,
    /// Shows the given card to the other players and discards it.
    Show(usize),
    ShowPile(u8, Vec<AustraliaCard>, Vec<char>),
    DiscardRequest,
    /// Discards the card in the players hand at that given index.
    Discard(usize),
    ScoreActivityQuery(Vec<AustralianActivity>),
    ScoreActivity(Option<AustralianActivity>),
    ReassignHand(Vec<AustraliaCard>),
    WaitingForPlayers,
    WaitingForPlayer,
    Connected(u8),
    UnexpectedMessage,
    Resend,
    Sync(AustraliaPlayer),
    NewRound,
    LobbyFull,
    FinalResult(u8, Vec<(u8, Scoring)>),
}

/// Messages passed between [`tui`] and
/// this crate
#[derive(Debug, Clone)]
pub enum Message {
    WaitingForPlayers,
    ReadyCheck,
    Ready,
    NotReady,
    Deal(AustraliaCard),
    DiscardQuery,
    Discard(AustraliaCard, usize),
    ShowQuery,
    Show(AustraliaCard, usize),
    ShowOtherHand(usize, Vec<AustraliaCard>, Vec<char>),
    ReassignHand(Vec<AustraliaCard>),
    Sync(AustraliaPlayer),
    Ok,
    ScoreActivityQuery(Vec<AustralianActivity>),
    ScoreActivity(Option<AustralianActivity>),
    NewRound,
    Exit,
    FinalResult(u8, Vec<(u8, Scoring)>),
}

impl TryInto<BackendEvent> for Event {
    type Error = ();
    fn try_into(self) -> Result<BackendEvent, Self::Error> {
        match self {
            Self::Connected(uid) => Ok(BackendEvent::Connected(uid)),
            Self::UnexpectedMessage => Ok(BackendEvent::UnexpectedMessage),
            Self::Resend => Ok(BackendEvent::Resend),
            _ => Err(()),
        }
    }
}

impl Into<Vec<u8>> for Event {
    fn into(self) -> Vec<u8> {
        serde_json::to_string(&self).unwrap().into_bytes()
    }
}
impl From<BackendEvent> for Event {
    fn from(value: BackendEvent) -> Self {
        match value {
            BackendEvent::Connected(uid) => Self::Connected(uid),
            BackendEvent::UnexpectedMessage => Self::UnexpectedMessage,
            BackendEvent::Resend => Event::Resend,
        }
    }
}

impl GameEvent for Event {
    fn requires_response(&self) -> bool {
        match self {
            Event::ReadyCheck => true,
            Event::Deal(_) => true,
            Event::Hand(_) => true,
            Event::DiscardRequest => true,
            Event::ShowRequest => true,
            Event::ReassignHand(_) => true,
            Event::ScoreActivityQuery(_) => true,
            Event::Sync(_) => true,
            _ => false,
        }
    }
}

impl UiMessage for Message {}
