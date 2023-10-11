pub mod card;
pub mod drawable;
pub mod event;
pub mod hand;
pub mod player;
pub mod rules;
pub mod session;
use crate::engine::session::Lobby;

use self::event::Event;
use self::session::LobbyInterface;
use self::session::PlayerFromTcpStream;
use player::TcpPlayer;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::{net::TcpListener, net::TcpStream};
use tokio::net::TcpStream as TokioTcpStream;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
#[derive(Debug)]
enum Cmd {
    Add { user: TcpStream },
}
#[derive(Debug)]
pub enum EngineError {
    NoSuchPlayer,
}

/// Wait for tcp connections pass to adder
async fn tcp_listener(listener: TcpListener, tx: mpsc::Sender<Cmd>) {
    loop {
        println!("Waiting");
        for stream in listener.incoming() {
            println!("{:?}", stream);
            let stream = stream.unwrap();
            match tx.send(Cmd::Add { user: stream }).await {
                Ok(_) => {
                    println!("Message sent!");
                }
                Err(e) => {
                    eprintln!("Could not send add user to {:?}, error code : {:?}", tx, e);
                }
            }
        }
    }
}
type DefaultLobby = Lobby<rules::Austrailia<4>, 4>;
pub async fn manager(listener: TcpListener) {
    println!("In manager");
    let (tx, rx) = mpsc::channel::<Cmd>(32);
    tokio::spawn(async move {
        tcp_listener(listener, tx).await;
    });

    //let manager = Arc::new(Mutex::new(RefCell::new(UserManager::new())));
    let (event_tx, event_rx) = mpsc::channel(32);

    let lobby: Arc<Mutex<RefCell<DefaultLobby>>> =
        Arc::new(Mutex::new(RefCell::new(session::Lobby::new(0, event_rx))));
    // Start the lobby
    tokio::spawn(DefaultLobby::start(lobby.clone()));
    // Does not return until the program exists, basically a block until exit
    tcp_manager(rx, lobby.clone(), event_tx).await;
}

async fn monitor<T: session::LobbyInterface>(
    manager: Arc<Mutex<RefCell<T>>>,
    uid: usize,
    mut rx: broadcast::Receiver<player::Message>,
    tx: mpsc::Sender<(usize, Event)>,
) {
    println!("In monitor for {:?}", rx);
    loop {
        match rx.recv().await {
            Ok(message) => match message {
                player::Message::Recived { event, user } => match event {
                    Err(_) => {
                        break;
                    }
                    Ok(msg) => {
                        let tx_clone = tx.clone();
                        tokio::spawn(async move {
                            tx_clone
                                .send((uid, msg))
                                .await
                                .map_err(|e| {
                                    eprintln!("{:?}", e);
                                })
                                .unwrap();
                        });
                    }
                },
            },
            _ => {
                break;
            }
        };
    }
    println!("Closed");
    let _ = manager.lock().unwrap().borrow_mut().disconnect(uid);
}
async fn tcp_manager<
    const CAPACITY: usize,
    T: LobbyInterface + PlayerFromTcpStream<32, CAPACITY> + 'static + std::marker::Send,
>(
    mut rx: mpsc::Receiver<Cmd>,
    mgr: Arc<Mutex<RefCell<T>>>,
    event_tx: mpsc::Sender<(usize, Event)>,
) {
    // Manage incoming tcp connections
    while let Some(message) = rx.recv().await {
        let manager_clone = mgr.clone();
        let mut new_monitor = None;
        match message {
            Cmd::Add { user } => {
                let user = user;
                new_monitor = Some(
                    (manager_clone)
                        .lock()
                        .unwrap()
                        .borrow_mut()
                        .add::<TcpPlayer<CAPACITY, player::Whole>, std::net::TcpStream>(user),
                );
            }
            _ => {}
        }
        println!("{:?}", new_monitor);
        match new_monitor {
            Some((uid, channel)) => {
                let event_tx_clone = event_tx.clone();
                tokio::spawn(async move {
                    monitor(manager_clone, uid, channel, event_tx_clone).await;
                });
            }
            _ => {}
        }
    }
}
