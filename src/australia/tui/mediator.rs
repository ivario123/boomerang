/**
 * asd
 */
use std::sync::Arc;

use log::{error, info};
use tokio::sync::{broadcast, RwLock};
use tui::{
    tui::{
        popup::{self, info::Info, select::Select, Popup},
        Tui, TuiMonitor,
    },
    ui::Hand,
};

use crate::{
    australia::protocol::Message,
    australia::rules::{
        cards::{AustraliaCard, AustralianActivity, Card},
        AustraliaPlayer,
    },
};

use super::{
    map::australia::Map,
    pages::{main_page::DefaultMainPage, map_page, score_popup::Score, show_page::ShowPage},
    ScoreList,
};

#[async_trait::async_trait]
impl TuiMonitor<Message, Info, Select>
    for Tui<
        DefaultMainPage<AustraliaCard, AustraliaPlayer>,
        map_page::DefaultTuiMap<Map, ScoreList>,
        ShowPage<AustraliaCard, AustraliaPlayer>,
        Info,
        Select,
        Score,
    >
{
    /// Opens up a input box, this allows the user to select an option
    async fn select(page: Arc<RwLock<Box<Self>>>, mut popup: Select) {
        info!("Showing query prompt");
        page.write().await.cleanup_popup();
        while let true = page.write().await.showing_popup() {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
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
    /// Opens up a info box, this will be cleared by any button press
    async fn info(page: Arc<RwLock<Box<Self>>>, mut popup: Info) {
        info!("Showing info pup-up");
        page.write().await.cleanup_popup();
        while let true = page.write().await.showing_popup() {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
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
    /// Manages message passing between the [`player`](super::player) and the TUI.
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
                    let (write_part, _read_part) = broadcast::channel(32);
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
                    let (write_part, mut read_part) = broadcast::channel(32);
                    let num_options = options.len();
                    // Cloning here is fin since the vector is small
                    let mut selectable: Vec<String> =
                        AustralianActivity::to_string_vec(options.clone());
                    let mut locked = page.write().await;
                    let mut cards = locked.main_page().get_hand().get_hand();
                    cards.extend(locked.main_page().get_show().get_hand());

                    let mut dict: std::collections::HashMap<AustralianActivity, usize> =
                        std::collections::HashMap::new();
                    for card in cards {
                        if let Some(activity) = card.activity() {
                            let count = dict.entry(activity).or_insert(0);
                            *count += 1;
                        }
                    }
                    // Avoid deadlocking our selves
                    drop(locked);
                    for (text, activity) in selectable.iter_mut().zip(options.clone()) {
                        if let Some(&count) = dict.get(&activity) {
                            let intermediate = format!("\n({})", count);
                            *text = text.to_owned() + &intermediate;
                        } else {
                            let intermediate = format!("\n({})", 0);
                            *text = text.to_owned() + &intermediate;
                        }
                    }
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
                    let _new_player = page.write().await.paginate().replace_into(ShowPage::new(
                        uid,
                        AustraliaPlayer::new(0).set_cards(cards),
                        visited,
                    ));
                }
                Message::ReassignHand(cards) => {
                    let mut new_hand: AustraliaPlayer = AustraliaPlayer::new(0);
                    for card in cards {
                        new_hand.add_card(card);
                    }
                    page.write().await.main_page().reassign_hand(new_hand);

                    info!("Trying to show waiting for swapped hands dialog");
                    let (write_part, _read_part) = broadcast::channel(32);
                    let popup = Info::new(write_part, "Swapped hands!".to_owned());
                    let page_clone = page.clone();
                    tokio::spawn(async move { Self::info(page_clone.clone(), popup).await });
                }
                Message::Sync(mut player) => {
                    let hand = AustraliaPlayer::new(0).set_cards(player.get_hand());
                    let scores = player.scores();
                    let mut locked_page = page.write().await;
                    locked_page
                        .paginate()
                        .map_page()
                        .replace_score(ScoreList(scores));
                    let mut discard = player.get_discard();
                    discard.extend(player.get_show());
                    let discard = AustraliaPlayer::new(0).set_cards(discard);
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
                Message::FinalResult(uid, scores) => {
                    info!("Game is now over");
                    info!("Trying to show the score dialog");
                    let score = Score::new(uid, scores);
                    let mut locked = page.write().await;
                    let _ = transmit.send(Message::Exit);
                    locked.final_result(score);
                    return;
                }
                Message::NewRound => {
                    info!("new round");
                    info!("Trying to show new round dialog");
                    let (write_part, _read_part) = broadcast::channel(32);
                    let popup = Info::new(write_part, "Starting a new round, score for previous round can be seen on the map page".to_owned());
                    let page_clone = page.clone();
                    tokio::spawn(async move { Self::info(page_clone.clone(), popup).await });
                }
                Message::WaitingForPlayers => {
                    info!("Waiting for players");
                    {
                        let page_write = page.write().await;
                        if page_write.showing_popup() {
                            continue;
                        }
                    }
                    info!("Trying to show waiting for players dialog");
                    let (write_part, _read_part) = broadcast::channel(32);
                    let popup = Info::new(write_part, "Waiting for players".to_owned());
                    let page_clone = page.clone();
                    tokio::spawn(async move { Self::info(page_clone.clone(), popup).await });
                }
                Message::Exit => {
                    // Does not matter if this produces an error the program is shutting down
                    let _ = transmit.send(Message::Exit);
                    return;
                }
                _ => {}
            }
        }
    }
}
