pub mod card;
pub mod event;
pub mod hand;
pub mod player;
pub mod rules;
pub mod session;
use self::player::PlayerError;
use card::AustraliaCards;
use event::Event;
use player::{Player, TcpPlayer};
use std::cell::{Ref, RefCell};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{net::TcpListener, net::TcpStream, vec::Vec};
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::time::interval;
use tokio::{task, time};
#[derive(Debug)]
enum Cmd {
    Add { user: TcpStream },
    Remove { user: TcpStream },
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
}

type UsersRef = Arc<Mutex<Vec<Box<RefCell<dyn Player>>>>>;


impl UserManager {
    pub async fn add(
        &mut self,
        mut rx: mpsc::Receiver<Cmd>,
        sender: broadcast::Sender<player::Message>,
    ) {
        let mut user_counter: usize = 0;
        let interval = Duration::from_secs(10);
        let user_ref = Arc::new(self.users);
        loop {
            let mut new_monitor = false;
            match rx.recv().await {
                Some(stream) => match stream {
                    Cmd::Add { user } => {
                        let mut users = self.users.lock().unwrap();
                        users.push(Box::new(RefCell::new(TcpPlayer::new(
                            user,
                            user_counter.clone(),
                            sender_clone,
                        ))));
                        user_counter = user_counter + 1;
                        println!("Added new user, current list of users : {:?}", users);
                        self.users.lock();
                        new_monitor = true;
                    }
                    _ => {}
                },
                _ => {}
            }
            if new_monitor {
                tokio::spawn(monitor(
                    user_ref.clone(),
                    (user_counter - 1).clone(),
                    interval.clone(),
                ));
            }
        }
    }

    pub fn new() -> Self {
        Self {
            users: Mutex::new(Vec::new()),
        }
    }
}

pub async fn manager(listener: TcpListener) {
    println!("In manager");
    let (tx, rx) = mpsc::channel::<Cmd>(32);
    tokio::spawn(tcp_listener(listener, tx));
    let mut manager = UserManager::new();
    manager.add(rx).await;
    loop {}
}

#[cfg(test)]
mod engine_test {

    #[test]
    fn test_start() {}
}
