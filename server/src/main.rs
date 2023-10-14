mod engine;
use tokio;

use std::net::TcpListener;

use crate::engine::rules::Austrailia;

#[tokio::main]
async fn main() {
    let listener = match TcpListener::bind("127.0.0.1:2047") {
        Ok(val) => val,
        Err(e) => {
            println!("{:?}", e);
            panic!();
        }
    };
    type rules = Austrailia<4>;
    engine::manager::<rules,4,4>(listener).await;
    println!("Hello world");
    loop {}
}
