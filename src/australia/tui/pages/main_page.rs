//! Defines the main page.
//! 
//! This is where the player can select cards to discard or cards
//! to show.
//! They can also browse their show hand.

use std::marker::PhantomData;

use log::info;
use ratatui::{
    prelude::{Backend, Constraint, Direction, Layout, Rect},
    style::Color,
    Frame,
};
use tokio::sync::broadcast;

use tui::{
    tui::{
        controls::{Controls, EventApi},
        TuiPage,
    },
    ui::{Card, Hand},
};

use crate::australia::{protocol::Message, rules::cards::AustraliaCard};


#[derive(Debug)]
pub enum Error {
    /// There is already a pending action
    PendingAction,
}
pub trait CardArea<C: Card> {
    /// Goes to the next card in the series
    fn increment(&mut self);
    /// Goes to the previous card in the series
    fn decrement(&mut self);
    /// Returns the current card pointer
    fn card_ptr(&mut self) -> &mut usize;
    /// Draws the card to the terminal
    fn draw<B: Backend>(&mut self, frame: &mut Frame<B>, block: Rect, title: &str, outline: Color);
}

pub struct MainPage<C: Card, H: Hand<C> + CardArea<C>> {
    hand: H,
    discard_pile: H,
    card: PhantomData<C>,
    focused: bool,
    title: String,
    feedback_channel: Option<broadcast::Sender<Message>>,
    requested_action: Option<Message>,
}
impl<C: Card, H: Hand<C> + CardArea<C>> MainPage<C, H> {
    pub fn new() -> Self {
        Self {
            hand: H::new(),
            discard_pile: H::new(),
            card: PhantomData,
            title: "Game".to_owned(),
            focused: false,
            feedback_channel: None,
            requested_action: None,
        }
    }
    pub fn get_hand(&self) -> &H{
        &self.hand
    }
    pub fn get_show(&self) -> &H{
        &self.discard_pile
    }
}

impl<C: Card, H: Hand<C> + CardArea<C> + std::fmt::Debug> MainPage<C, H> {
    pub fn add_card(&mut self, card: C) {
        self.hand.add_card(card);
    }
    pub fn request(
        &mut self,
        event: Message,
        channel: broadcast::Sender<Message>,
    ) -> Result<(), Error> {
        let pending_event = match self.requested_action {
            Some(_) => return Err(Error::PendingAction),
            _ => &mut self.requested_action,
        };
        let feedback_channel = match self.feedback_channel {
            Some(_) => return Err(Error::PendingAction),
            _ => &mut self.feedback_channel,
        };
        info!("Enqueued event : {:?} for default main page ", event);
        *pending_event = Some(event);
        *feedback_channel = Some(channel);
        Ok(())
    }
    pub fn reassign_hand(&mut self, hand: H) {
        info!("Replacing hand with {:?} hand", hand);
        self.hand = hand;
        info!("Done!");
    }
    pub fn reassign_show(&mut self, discard_pile: H) {
        info!("Replacing show pile with {:?} hand", discard_pile);
        self.discard_pile = discard_pile;
        info!("Done!");
    }
}

impl<H: Hand<AustraliaCard> + CardArea<AustraliaCard>> EventApi
    for MainPage<AustraliaCard, H>
{
    fn handle_input(&mut self, control: Controls) {
        let focused = match self.focused {
            true => &mut self.discard_pile,
            false => &mut self.hand,
        };

        match control {
            Controls::Left => focused.decrement(),
            Controls::Right => focused.increment(),
            Controls::Down | Controls::Up => self.focused = !self.focused,
            Controls::Enter => {
                if let Some(pending_event) = self.requested_action.clone() {
                    if let Some(channel) = self.feedback_channel.clone() {
                        let action = match pending_event {
                            Message::DiscardQuery => {
                                let ptr = *self.hand.card_ptr();
                                let card = self.hand.discard_card(ptr);
                                self.discard_pile.add_card(card);
                                self.requested_action = None;
                                self.feedback_channel = None;
                                Message::Discard(card, ptr)
                            }
                            Message::ShowQuery => {
                                let ptr = *self.hand.card_ptr();
                                let card = self.hand.discard_card(ptr);
                                self.discard_pile.add_card(card);
                                self.requested_action = None;
                                self.feedback_channel = None;
                                Message::Show(card, ptr)
                            }
                            _ => return, // Handle any other cases here
                        };
                        let channel = channel.clone();
                        tokio::spawn(async move {
                            channel.send(action).unwrap();
                        });
                    }
                }
            }
            _ => {}
        }
    }
}

impl<H: Hand<AustraliaCard> + CardArea<AustraliaCard>> TuiPage
    for MainPage<AustraliaCard, H>
{
    fn draw<B: Backend>(&mut self, frame: &mut Frame<B>, block: Rect) {
        let layout: std::rc::Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(45),
                    Constraint::Percentage(5),
                    Constraint::Percentage(45),
                ]
                .as_ref(),
            )
            .split(block);
        let (hand, board) = match self.focused {
            true => (Color::DarkGray, Color::White),
            false => (Color::White, Color::DarkGray),
        };
        self.hand.draw(frame, layout[0], "Hand", hand);

        self.discard_pile.draw(frame, layout[2], "Board", board);
    }
    fn set_title(&mut self, title: String) {
        self.title = title
    }
    fn get_title(&self) -> &str {
        &self.title
    }
}
