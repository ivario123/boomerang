use server::engine::rules::{AustraliaCard, Event};
use std::io::{Read, Write};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
#[tokio::main]
async fn main() {
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
        println!("Recived : {:?}", event);
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
            Event::ShowPile(idx,cards) => {
                println!("Player {:?} is showing {:?}", idx,cards);
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
