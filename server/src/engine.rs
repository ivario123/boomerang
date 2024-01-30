pub mod event;
pub mod player;
pub mod rules;
pub mod session;
use crate::engine::session::Lobby;

use self::event::GameEvent;
use self::player::Message;
use self::rules::{Instantiable, RuleEngine};
use self::session::{LobbyInterface, PlayerFromTcpStream, SessionError};
use std::cell::RefCell;
use std::sync::Arc;
use std::{net::TcpListener, net::TcpStream};
use tokio::sync::{broadcast, mpsc, Mutex};

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
pub async fn manager<
    Rules: RuleEngine + Instantiable + Send + 'static,
    const BUFFER_SIZE: usize,
    const CAPACITY: usize,
>(
    listener: TcpListener,
) where
    Lobby<Rules, CAPACITY>: PlayerFromTcpStream<CAPACITY, BUFFER_SIZE, Rules::Event>,
{
    println!("In manager");
    let (tx, rx) = mpsc::channel::<Cmd>(32);
    tokio::spawn(async move {
        tcp_listener(listener, tx).await;
    });

    //let manager = Arc::new(Mutex::new(RefCell::new(UserManager::new())));
    let (event_tx, event_rx) = mpsc::channel(32);

    let lobby: Arc<Mutex<RefCell<Lobby<Rules, CAPACITY>>>> =
        Arc::new(Mutex::new(RefCell::new(session::Lobby::new(0, event_rx))));
    // Start the lobby
    tokio::spawn(Lobby::<Rules, CAPACITY>::start(lobby.clone()));
    // Does not return until the program exists, basically a block until exit
    tcp_manager(rx, lobby.clone(), event_tx).await;
}

async fn monitor<Event: GameEvent, T: session::LobbyInterface<Event>>(
    manager: Arc<Mutex<RefCell<T>>>,
    uid: usize,
    mut rx: broadcast::Receiver<player::Message<Event>>,
    tx: mpsc::Sender<(usize, Event)>,
) {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
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
            _e => {}
        };
    }
    println!("Closed");
    let _ = manager.lock().await.borrow_mut().disconnect(uid);
}

fn add_player<
    Event: GameEvent + 'static,
    const BUFFER_SIZE: usize,
    const CAPACITY: usize,
    T: LobbyInterface<Event>
        + PlayerFromTcpStream<BUFFER_SIZE, CAPACITY, Event>
        + 'static
        + std::marker::Send,
>(
    player: Result<(usize, broadcast::Receiver<Message<Event>>), SessionError>,
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
    }
}
async fn tcp_manager<
    Event: GameEvent + 'static,
    const CAPACITY: usize,
    const BUFFER_SIZE: usize,
    T: LobbyInterface<Event>
        + PlayerFromTcpStream<CAPACITY, BUFFER_SIZE, Event>
        + 'static
        + std::marker::Send,
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

                let user = borrowed_manager.add(user);

                add_player(user, manager.clone(), event_tx.clone());
            }
        }
    }
}
