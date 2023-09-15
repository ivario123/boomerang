pub mod card;
pub mod event;
pub mod hand;
pub mod player;
pub mod rules;
pub mod session;
use event::Event;

use card::AustraliaCards;
use player::{Player, TcpPlayer};
use std::{net::TcpListener, net::TcpStream, vec::Vec};

pub async fn manager(listener: TcpListener) {
    loop {
        println!("Waiting");
        for stream in listener.incoming() {
            println!("{:?}", stream);
            let stream = stream.unwrap();

            // Create  anew player
            let mut player = TcpPlayer::new(stream, 1);
            println!("{:?}", player);

            // Read data from player
            let ret = player.recive().await;
            // Send draw deal card to player
            player
                .send(Event::Deal(AustraliaCards::ThePinnacles))
                .unwrap();
            println!("{:?}", ret);
            let ret = player.recive().await;
            println!("{:?}", ret);
        }
    }
}

#[cfg(test)]
mod engine_test {

    #[test]
    fn test_start() {}
}
