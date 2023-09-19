pub mod card;
pub mod event;
pub mod hand;
pub mod player;
pub mod rules;
pub mod session;
use crate::engine::session::Lobby;

use self::event::Event;
use self::player::Splittable;
use self::rules::RuleSet;
use self::session::LobbyInterface;
use self::session::PlayerFromTcpStream;
use player::New;
use player::Reciver;
use player::{Player, TcpPlayer};
use session::Session;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::{net::TcpListener, net::TcpStream, vec::Vec};
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

/// Manages all lobby less users
struct UserManager {
    /// The user is represented as a list of heap allocated users and
    /// their
    users: Mutex<Vec<Box<RefCell<dyn Player>>>>,
    user_counter: usize,
}
/*
impl session::Session for UserManager {
    type Error = EngineError;
    fn new() -> Self {
        Self {
            users: Mutex::new(Vec::new()),
            user_counter: 0,
        }
    }
    fn delete(&mut self, uid: usize) -> Result<Box<RefCell<dyn Player>>, EngineError> {
        let mut idx = None;
        let mut users = self.users.lock().unwrap();
        for (id, user) in users.iter().enumerate() {
            idx = match user.borrow().getid() == uid {
                true => Some(id),
                _ => idx,
            };
        }

        let ret = match idx {
            Some(id) => {
                let user = users.remove(id);
                Ok(user)
            }
            None => Err(EngineError::NoSuchPlayer),
        };
        println!("Deleted : {:?}", ret);
        return ret;
    }
    fn add<P: Player + Splittable<dyn Reciver> + 'static, T: New<P>>(
        &mut self,
        user: T,
    ) -> (usize, broadcast::Receiver<player::Message>) {
        let mut users = self.users.lock().unwrap();
        let user = user.new(self.user_counter);

        let (user, mut read_part) = user.split();
        let monitor = read_part.subscribe().unwrap();

        // Spawn a reading task
        tokio::spawn(async move { read_part.recive().await });

        // Place user on the heap.
        users.push(Box::new(RefCell::new(user)));

        self.user_counter = self.user_counter + 1;

        drop(users);

        return (self.user_counter - 1, monitor);
    }
}
*/
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
                            tx_clone.send((uid, msg)).await.map_err(|e| {
                                eprintln!("{:?}", e);
                            }).unwrap();
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
pub async fn manager(listener: TcpListener) {
    println!("In manager");
    let (tx, rx) = mpsc::channel::<Cmd>(32);
    tokio::spawn(async move {
        tcp_listener(listener, tx).await;
    });

    //let manager = Arc::new(Mutex::new(RefCell::new(UserManager::new())));
    let (event_tx, event_rx) = mpsc::channel(32);
    let lobby: Arc<Mutex<RefCell<Lobby<rules::Austrailia<4>, 4>>>> =
        Arc::new(Mutex::new(RefCell::new(session::Lobby::new(0, event_rx))));

    tcp_manager(rx, lobby.clone(), event_tx).await;
}
async fn tcp_manager<T: LobbyInterface + PlayerFromTcpStream<32> + 'static + std::marker::Send>(
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
                let user: std::net::TcpStream = user;
                new_monitor = Some(
                    (manager_clone)
                        .lock()
                        .unwrap()
                        .borrow_mut()
                        .add::<TcpPlayer<32, player::Whole>, std::net::TcpStream>(user),
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
#[cfg(test)]
mod engine_test {

    #[test]
    fn test_start() {}
}
