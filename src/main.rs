mod rules;
extern crate clap;
use std::net::TcpListener;

use clap::{command, Parser, ValueEnum};
use rules::{cards::AustraliaCard, Event};
use server::engine;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::rules::Australia;

#[derive(Debug,Clone, Copy, ValueEnum)]
pub enum Mode {
    Server,
    Client,
}

#[derive(Parser)] // requires `derive` feature
#[command(
    author = "Ivar Jönsson <ivajns-9@student.ltu.se>",
    version = "0.0.1",
    about = "Boomerang Australia",
    long_about = "Implements a generic boomerang client and server with pre defined rules for the boomerang
australia game."
)]
struct Args {
    #[arg(short = 'm')]
    mode: Mode,
}

async fn player_main() {
    let mut stream = match TcpStream::connect("127.0.0.1:2047").await {
        Ok(val) => val,
        Err(e) => {
            println!("{:?}", e);
            panic!();
        }
    };

    let (mut read_part, mut write_part) = stream.split();
    let mut send = |msg: Event| {
        let to_send: &Vec<u8> = &msg.into();
        async_std::task::block_on(write_part.write(to_send)).unwrap();
    };
    let mut read = || -> Event {
        let mut buff = vec![0; 256];
        async_std::task::block_on(read_part.read(&mut buff)).unwrap();
        while let Some(&0) = buff.last() {
            buff.pop();
        }
        serde_json::from_slice::<Event>(&buff).unwrap()
    };
    let mut hand = Vec::<AustraliaCard>::new();
    loop {
        let event = read();
        println!("Received : {:?}", event);
        match event {
            Event::ReadyCheck => {
                send(Event::Accept);
                println!("Responded with {:?}", Event::Accept);
            }
            Event::Deal(card) => {
                hand.push(card);
                send(Event::Accept);
                println!("Responded with {:?}", Event::Accept);
            }
            Event::UnexpectedMessage => {
                send(Event::Accept);
                println!("Responded with {:?}", Event::Accept);
            }
            Event::DiscardRequest => {
                hand.remove(0);
                send(Event::Discard(0));
                println!("Responded with {:?}", Event::Discard(0));
            }
            Event::ShowRequest => {
                hand.remove(0);
                send(Event::Show(0));
                println!("Responded with {:?}", Event::Show(0));
            }
            Event::ShowPile(idx, cards) => {
                println!("Player {:?} is showing {:?}", idx, cards);
            }
            Event::ReassignHand(new_hand) => {
                println!("Replacing {:?} with {:?}", hand, new_hand);
                hand = new_hand;
                send(Event::Accept);
                println!("Responded with {:?}", Event::Accept);
            }
            _ => {}
        }
    }
}

async fn server_main() {
    println!("Running as server");
    let listener = match TcpListener::bind("127.0.0.1:2047") {
        Ok(val) => val,
        Err(e) => {
            println!("{:?}", e);
            panic!();
        }
    };
    type Rules = Australia<4, 2>;
    engine::manager::<Rules, 4, 4>(listener).await;
    println!("Hello world");
    loop {}
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    println!("{:?}",args.mode);
    match args.mode {
        Mode::Server => server_main().await,
        Mode::Client => player_main().await,
    }
}
