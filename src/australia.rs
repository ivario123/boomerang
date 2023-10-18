pub mod mainpage;
pub mod mappage;
pub mod showpage;

use async_std::channel::{self, Recv};
use log::{error, info};
use ratatui::{
    prelude::{Backend, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex, RwLock};
use tui::{
    maps::boomerang_australia::Map,
    tui::{
        popup::{self, info::Info, select::Select, Popup},
        Tui, TuiMonitor, TuiPage,
    },
    ui::{Card, Hand, UiElement, UiMessage},
};

use crate::{
    read_event,
    rules::{
        cards::{AustraliaCard, AustralianActivity, Card as CardTrait},
        AustraliaPlayer,
    },
};

use self::{
    mainpage::{CardArea, DefaultMainPage},
    showpage::ShowPage,
};

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
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Percentage((100 / 5) as u16); 5].as_ref())
                .split(*area)
                .to_vec();

            let mut paragraphs = Vec::new();
            paragraphs.push(Paragraph::new(format!(
                "Site : {:?} | # : {:?}",
                card.to_char(),
                CardTrait::number(card)
            )));
            paragraphs.push(Paragraph::new(format!("Region : {:?}", card.region())));
            if let Some(collection) = card.collection() {
                paragraphs.push(Paragraph::new(format!("Collection : {:?}", collection)));
            }
            if let Some(animal) = card.animal() {
                paragraphs.push(Paragraph::new(format!("Animal : {:?}", animal)));
            }
            if let Some(activity) = card.activity() {
                paragraphs.push(Paragraph::new(format!("Activity : {:?}", activity)));
            }

            frame.render_widget(rect, *area);
            for (area, par) in layout.iter().zip(paragraphs) {
                frame.render_widget(par, *area);
            }
        }
    }
}

#[async_trait::async_trait]
impl TuiMonitor<Message, Info, Select>
    for Tui<
        DefaultMainPage<AustraliaCard, AustraliaPlayer>,
        mappage::DefaultTuiMap<Map>,
        crate::australia::showpage::ShowPage<AustraliaCard, AustraliaPlayer>,
        Info,
        Select,
    >
{
    async fn select(page: Arc<RwLock<Box<Self>>>, mut popup: Select) {
        info!("Showing query prompt");
        page.write().await.cleanup_popup();
        let mut channel = {
            let mut page_write = page.write().await;
            let channel = popup.subscribe();
            // No popup is showing, unwrapping is fine
            page_write.show_query(popup).unwrap();
            channel
        };
        loop {
            match channel.recv().await {
                Ok(popup::Message::Close) => {
                    page.write().await.clear_popup();
                    return;
                }
                _ => {}
            }
        }
    }
    async fn info(page: Arc<RwLock<Box<Self>>>, mut popup: Info) {
        info!("Showing info pupup");
        page.write().await.cleanup_popup();
        while let true = page.write().await.showing_popup() {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
        let mut channel = {
            let mut page_write = page.write().await;
            let channel = popup.subscribe();
            // No popup is showing, unwrapping is fine
            page_write.show_info(popup).unwrap();
            channel
        };
        loop {
            match channel.recv().await {
                Ok(popup::Message::Close) => {
                    let mut page_write = page.write().await;
                    page_write.clear_popup();
                    drop(page_write);
                    info!("Closed the popup");
                    return;
                }
                _ => {}
            }
        }
    }

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
                // ================================================================================
                //                          Requires Manual Intervention
                // ================================================================================

                // -------------------------            Queries           -------------------------
                Message::DiscardQuery => {
                    info!("Trying to show waiting for discard dialog");
                    let (write_part, read_part) = broadcast::channel(32);
                    let popup = Info::new(
                        write_part,
                        "Now your hand is full, time to discard a card".to_owned(),
                    );
                    let page_clone = page.clone();
                    tokio::spawn(async move { Self::info(page_clone.clone(), popup).await });
                    page.write()
                        .await
                        .main_page()
                        .request(Message::DiscardQuery, transmit.clone())
                        .unwrap();
                }
                Message::ShowQuery => {
                    let (write_part, _) = broadcast::channel(32);
                    let popup = Info::new(
                        write_part,
                        "Select a card to show to the other players".to_owned(),
                    );
                    let page_clone = page.clone();
                    tokio::spawn(async move { Self::info(page_clone.clone(), popup).await });
                    page.write()
                        .await
                        .main_page()
                        .request(Message::ShowQuery, transmit.clone())
                        .unwrap();
                }
                Message::ReadyCheck => {
                    let (write_part, mut read_part) = broadcast::channel(32);
                    let popup = Select::new(
                        write_part,
                        vec!["Yes".to_owned(), "Not yet".to_owned()],
                        "Do you want to start the game?".to_owned(),
                        20,
                        60,
                    );
                    tokio::spawn(Self::select(page.clone(), popup));

                    match read_part.recv().await {
                        Ok(popup::Message::Select(0)) => {
                            transmit.send(Message::Ready).unwrap();
                        }
                        _ => {
                            transmit.send(Message::NotReady).unwrap();
                        }
                    }
                }
                Message::ScoreActivityQuery(options) => {
                    transmit.send(Message::ScoreActivity(None)).unwrap();
                    continue;
                    // Felet ligger någon stans i spawnandet av dialogen, vet inte vad som går fel
                    // kan race conditions eller ett deadlock men tracet visar att 
                    // vi kanalen går ur scope innan vi får svar ifrån den, kan vara någon info
                    // dialog som kommer emellan vet inte.
                    let (write_part, mut read_part) = broadcast::channel(32);
                    let num_options = options.len();
                    // Cloning here is fin since the vector is small
                    let mut selectable: Vec<String> =
                        AustralianActivity::to_string_vec(options.clone());
                    selectable.push("Do not score anything".to_owned());
                    let popup = Select::new(
                        write_part,
                        selectable,
                        "What activity do you want to score this round?".to_owned(),
                        20,
                        90,
                    );
                    let page_clone = page.clone();
                    tokio::spawn(async move { Self::select(page_clone.clone(), popup).await });
                    loop {
                        match read_part.recv().await {
                            Ok(popup::Message::Select(x)) => {
                                transmit
                                    .send(Message::ScoreActivity(match x < num_options - 1 {
                                        true => Some(options[x]),
                                        false => None,
                                    }))
                                    .unwrap();
                                break;
                            }

                            Err(_) => {
                                error!("Frontend must have crashed");
                                transmit.send(Message::ScoreActivity(None)).unwrap();
                                return;
                            }
                            _ => {}
                        }
                    }
                }
                // ================================================================================
                //                              Automated responses
                // ================================================================================
                // -------------------------            Updates           -------------------------
                Message::Deal(card) => page.write().await.main_page().add_card(card),
                Message::ShowOtherHand(uid, cards, visited) => {
                    let mut new_player = page.write().await.paginate().replace_into(ShowPage::new(
                        uid,
                        AustraliaPlayer::new().set_cards(cards),
                        visited,
                    ));
                }
                Message::ReassignHand(cards) => {
                    let mut new_hand: AustraliaPlayer = AustraliaPlayer::new();
                    for card in cards {
                        new_hand.add_card(card);
                    }
                    page.write().await.main_page().reassign_hand(new_hand);

                    info!("Trying to show waiting for swapped hands dialog");
                    let (write_part, read_part) = broadcast::channel(32);
                    let popup = Info::new(write_part, "Swapped hands!".to_owned());
                    let page_clone = page.clone();
                    tokio::spawn(async move { Self::info(page_clone.clone(), popup).await });
                }
                Message::Sync(mut player) => {
                    let hand = AustraliaPlayer::new().set_cards(player.get_hand());

                    let mut discard = player.get_discard();
                    discard.extend(player.get_show());
                    let discard = AustraliaPlayer::new().set_cards(discard);
                    let mut locked_page = page.write().await;
                    locked_page.main_page().reassign_hand(hand);
                    locked_page.main_page().reassign_show(discard);
                    locked_page
                        .paginate()
                        .map_page()
                        .update_visited(player.privately_visited());
                    // Ensure that lock is freed before transmit
                    drop(locked_page);
                    transmit.send(Message::Ok).unwrap();
                }
                // -------------------------            Status           -------------------------
                Message::WaitingForPlayers => {
                    info!("Waiting for players");
                    {
                        let page_write = page.write().await;
                        if page_write.showing_popup() {
                            continue;
                        }
                    }
                    info!("Trying to show waiting for players dialog");
                    let (write_part, read_part) = broadcast::channel(32);
                    let popup = Info::new(write_part, "Waiting for players".to_owned());
                    let page_clone = page.clone();
                    tokio::spawn(async move { Self::info(page_clone.clone(), popup).await });
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
