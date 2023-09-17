mod engine;
use tokio;

use std::net::TcpListener;

#[tokio::main]
async fn main() {
    let listener = match TcpListener::bind("127.0.0.1:2047") {
        Ok(val) => val,
        Err(e) => {
            println!("{:?}", e);
            panic!();
        }
    };
    engine::manager(listener).await;
    println!("Hello world");
    loop {}
}
