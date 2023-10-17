pub mod mainpage;
pub mod mappage;

use ratatui::{
    prelude::{Backend, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders},
    Frame,
};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex, RwLock};
use tui::{
    maps::boomerang_australia::Map,
    tui::{Tui, TuiMonitor, TuiPage},
    ui::{Card, Hand, UiElement, UiMessage},
};

use crate::rules::{cards::AustraliaCard, AustraliaPlayer};

use self::mainpage::{CardArea, DefaultMainPage};

#[derive(Debug, Clone)]
pub enum Message {
    Deal(AustraliaCard),
    DiscardQuery,
    Discard(AustraliaCard, usize),
    ShowQuery,
    Show(AustraliaCard, usize),
    ReassignHand(Vec<AustraliaCard>),
    Exit,
}
impl UiMessage for Message {}

impl CardArea<AustraliaCard> for AustraliaPlayer
where
    Self: Hand<AustraliaCard>,
{
    fn increment(&mut self) {
        let count = self.hand_size();
        let ptr = self.card_ptr();
        if count > 0 && *ptr < count - 1 {
            *ptr += 1;
        }
    }

    fn decrement(&mut self) {
        let ptr = self.card_ptr();
        if *ptr > 0 {
            *ptr -= 1;
        }
    }

    fn card_ptr(&mut self) -> &mut usize {
        self.card_ptr()
    }

    fn draw<B: Backend>(&mut self, frame: &mut Frame<B>, block: Rect, title: &str, border: Color) {
        let disp_ptr = *self.card_ptr();
        let (cards, (last, count)) = self.get_cards::<4>(disp_ptr);

        let card_area = Block::default()
            .title(format!(
                "{} (showing {} - {}/{})",
                title,
                disp_ptr + 1,
                last,
                count
            ))
            .borders(Borders::all())
            .style(Style::default().fg(border));
        frame.render_widget(card_area, block);

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Percentage((100 / 4) as u16); 4].as_ref())
            .split(block);
        let num_cards = cards.len();

        for (idx, (area, card)) in layout.iter().zip(cards).enumerate() {
            let rect = Block::default()
                .title(card.get_name())
                .borders(Borders::all())
                .border_style(
                    Style::default().fg(match (idx + last - num_cards) == disp_ptr {
                        true => Color::Yellow,
                        false => Color::Gray,
                    }),
                );
            frame.render_widget(rect, *area);
        }
    }
}

#[async_trait::async_trait]
impl TuiMonitor<Message>
    for Tui<DefaultMainPage<AustraliaCard, AustraliaPlayer>, mappage::DefaultTuiMap<Map>>
{
    async fn monitor(
        page: Arc<RwLock<Box<Self>>>,
        mut channel: broadcast::Receiver<Message>,
        transmit: broadcast::Sender<Message>,
    ) {
        loop {
            // Poll for events every second
            let msg = channel.recv().await;
            if let Err(e) = msg {
                match e {
                    _ => return,
                }
            }
            let msg = msg.unwrap();
            match msg {
                Message::Deal(card) => page.write().await.main_page().add_card(card),
                Message::DiscardQuery => {
                    page.write()
                        .await
                        .main_page()
                        .request(Message::DiscardQuery, transmit.clone())
                        .unwrap();
                }
                Message::ShowQuery => {
                    page.write()
                        .await
                        .main_page()
                        .request(Message::ShowQuery, transmit.clone())
                        .unwrap();
                }
                Message::ReassignHand(cards) => {
                    let mut new_hand: AustraliaPlayer = AustraliaPlayer::new();
                    for card in cards {
                        new_hand.add_card(card);
                    }
                    page.write().await.main_page().reassign_hand(new_hand);
                }
                Message::Exit => {
                    // Does not matter if this produces an error the program is shutting down
                    transmit.send(Message::Exit);
                    return;
                }
                _ => {}
            }
        }
    }
}
