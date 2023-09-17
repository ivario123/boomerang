pub mod card;
pub mod event;
pub mod hand;
pub mod player;
pub mod rules;
pub mod session;
use self::player::PlayerError;
use async_recursion::async_recursion;
use card::AustraliaCards;
use event::Event;
use player::{Player, TcpPlayer};
use std::cell::{Ref, RefCell};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use std::{net::TcpListener, net::TcpStream, vec::Vec};
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::time::interval;
use tokio::{task, time};
use player::Reciver;
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
    user_counter: usize,
}

type UsersRef = Arc<Mutex<Vec<Box<RefCell<dyn Player>>>>>;

impl UserManager {
    async fn monitor(&mut self, reciver: broadcast::Receiver<player::Message>) {
        return;
        todo!();
    }
    pub fn new() -> Self {
        Self {
            users: Mutex::new(Vec::new()),
            user_counter: 0,
        }
    }
    pub fn add(&mut self, user: TcpStream) -> broadcast::Receiver<player::Message> {
        let mut users = self.users.lock().unwrap();
        let user = tokio::net::TcpStream::from_std(user).unwrap();
        let (mut read_part,user) = TcpPlayer::<32,player::Whole>::new(user, self.user_counter.clone()).split();
        let monitor = read_part.subscribe().unwrap();
        tokio::spawn(
            async  {
                read_part.recive().await
            }
        );



        users.push(Box::new(RefCell::new(user)));
        self.user_counter = self.user_counter + 1;
        println!("Added new user, current list of users : {:?}", users);
        drop(users);
        return monitor;
    }
}

struct Monitor {
    user_manager: Arc<UserManager>,
}
impl Monitor {}

async fn monitor(
    manager: Arc<Mutex<RefCell<UserManager>>>,
    mut rx: broadcast::Receiver<player::Message>,
) {
    println!("In monitor for {:?}", rx);
    while let Ok(message) = rx.recv().await {
        println!("Some message {:?}", message);
    }
    println!("Closed");
}
pub async fn manager(listener: TcpListener) {
    println!("In manager");
    let (tx, mut rx) = mpsc::channel::<Cmd>(32);
    tokio::spawn(async move {
        tcp_listener(listener, tx).await;
    });

    let mut manager = Arc::new(Mutex::new(RefCell::new(UserManager::new())));
    while let Some(message) = rx.recv().await {
        let mut manager_clone = manager.clone();
        let mut new_monitor = None;
        let rec = rx.recv().await;
        match rec {
            Some(stream) => {
                match stream {
                    Cmd::Add { user } => {
                        new_monitor = Some((manager).lock().unwrap().borrow_mut().add(user));
                    }
                    _ => {}
                };
            }
            _ => {}
        }
        println!("{:?}", new_monitor);
        match new_monitor {
            Some(channel) => {
                tokio::spawn(async move {
                    monitor(manager_clone, channel).await;
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
