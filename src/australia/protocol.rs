//! Defines communication message space for player and server.
//! 
//! The tcp protocol is defines as  [`Event`] and the intra app protocol
//! is defined in [`Message`].


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
    /// Server is ready to start
    /// 
    /// This requires a [`Accept`](Event::Accept) or a [`Deny`](Event::Deny) response
    /// any other response will be disregarded
    ReadyCheck,
    /// Response sent from player
    Accept,
    /// Response sent from player
    Deny,
    /// Server has dealt a card to the receiving player
    /// 
    /// This requires a [`Accept`](Event::Accept)  response
    /// any other response will be disregarded
    Deal(AustraliaCard),
    /// Asks the player to show a card
    /// 
    /// This requires a [`Show`](Event::Show(())) response.
    ShowRequest,
    /// Shows the given card to the other players and discards it.
    Show(usize),
    /// Shows the player what another player has in their show pile
    ShowPile(u8, Vec<AustraliaCard>, Vec<char>),
    /// Asks the player to discard a card
    /// 
    /// This requires a [`Show`](Event::Discard(())) response.
    DiscardRequest,
    /// Discards the card in the players hand at that given index.
    Discard(usize),
    /// Asks the player activity they want to score this turn
    /// 
    /// This requires a [`ScoreActivity`](Event::ScoreActivity(())) response.
    ScoreActivityQuery(Vec<AustralianActivity>),
    /// Scores the given activity if any
    ScoreActivity(Option<AustralianActivity>),
    /// Overwrites the current hand replacing it with a new one.
    ReassignHand(Vec<AustraliaCard>),
    /// Status message
    WaitingForPlayers,
    /// Here fore completeness sake
    /// 
    /// Maps from [`Connected`](BackendEvent::Connected(()))
    Connected(u8),

    /// Server did not expect that response
    UnexpectedMessage,
    /// Unused here fore completeness sake.
    Resend,
    /// Syncs player game data with the servers.
    /// 
    /// This requires a [`Accept`](Event::Accept)  response
    /// any other response will be disregarded
    Sync(AustraliaPlayer),
    /// Status message informs player that the new round has started
    NewRound,
    /// Status message informs players that the game cannot start yet
    LobbyFull,
    /// Status message informs players of game final result.
    FinalResult(u8, Vec<(u8, Scoring)>),
}

/// Messages passed between [`tui`] and
/// this crate
/// 
/// This is more or less a subset of [`Event`]
/// so for detailed explanations read the docs 
/// for [`Event`]
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
