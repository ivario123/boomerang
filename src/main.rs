use server::engine::event::Event;
use std::{
    io::{Read, Write},
    net::TcpStream,
};
fn main() {
    let mut stream = match TcpStream::connect("127.0.0.1:2047") {
        Ok(val) => val,
        Err(e) => {
            println!("{:?}", e);
            panic!();
        }
    };
    let msg = Event::Connected(0);
    let sending = serde_json::to_string(&msg).unwrap();
    for el in 0..10000 {
        println!("Doing things");
    }
    let to_send: &Vec<u8> = &msg.into();
    stream.write(to_send).unwrap();
    let mut buff = vec![0;100];
    stream.read(&mut buff).unwrap();
    while let Some(&0) = buff.last() {
        buff.pop();
    }
    let parsed_str = serde_json::from_slice::<Event>(&buff).unwrap();
    println!("Recived : {:?}", parsed_str);
}
