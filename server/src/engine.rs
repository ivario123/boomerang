pub mod card;
pub mod drawable;
pub mod event;
pub mod hand;
pub mod player;
pub mod rules;
pub mod session;
use crate::engine::session::Lobby;

use self::event::Event;
use self::player::Message;
use self::session::LobbyInterface;
use self::session::PlayerFromTcpStream;
use self::session::SessionError;
use player::TcpPlayer;
use std::cell::RefCell;
use std::sync::{Arc};
use tokio::sync::Mutex;
use std::{net::TcpListener, net::TcpStream};

use tokio::sync::broadcast;
use tokio::sync::mpsc;
#[derive(Debug)]
enum Cmd {
    Add { user: TcpStream },
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
    //println!("In monitor for {:?}", rx.resubscribe());
    loop {
        match rx.try_recv() {
            Ok(message) => {
                println!("{:?}", message);
                match message {
                    player::Message::Received { event, user: _ } => match event {
                        Err(e) => {
                            println!("Exiting the monitor for some reason {:?}", e);
                            break;
                        }
                        Ok(msg) => {
                            println!("monitor got {:?}", msg);
                            tx.send((uid, msg)).await.unwrap();
                            println!("sent");
                        }
                    },
                }
            }
            e => {}
        };
    }
    println!("Closed");
    let _ = manager.lock().await.borrow_mut().disconnect(uid);
}

fn add_player<
    const CAPACITY: usize,
    T: LobbyInterface + PlayerFromTcpStream<32, CAPACITY> + 'static + std::marker::Send,
>(
    player: Result<(usize, broadcast::Receiver<Message>), SessionError>,
    manager: Arc<Mutex<RefCell<T>>>,
    event_tx: mpsc::Sender<(usize, Event)>,
) {
    println!("hey, new player {:?}", player);
    match player {
        Ok((uid, channel)) => {
            println!("Spawning monitor for {:?} and {:?}", uid, channel);
            tokio::spawn(async move {
                monitor(manager, uid, channel, event_tx).await;
            });
        }
        Err(e) => {
            println!("{:?}", e);
        }
        _ => {
            // No new player to add
        }
    }
}
async fn tcp_manager<
    const CAPACITY: usize,
    T: LobbyInterface + PlayerFromTcpStream<32, CAPACITY> + 'static + std::marker::Send,
>(
    mut rx: mpsc::Receiver<Cmd>,
    manager: Arc<Mutex<RefCell<T>>>,
    event_tx: mpsc::Sender<(usize, Event)>,
) {
    // Manage incoming tcp connections
    while let Some(message) = rx.recv().await {
        match message {
            Cmd::Add { user } => {
                let cloned_manager = manager.clone();
                let locked_manager = cloned_manager.lock().await;
                let mut borrowed_manager = match locked_manager.try_borrow_mut() {
                    Ok(manager) => manager,
                    Err(_) => {
                        return;
                    }
                };

                let user = borrowed_manager
                    .add::<TcpPlayer<CAPACITY, player::Whole>, std::net::TcpStream>(user);

                add_player(user, manager.clone(), event_tx.clone());
            }
        }
    }
}
