mod australia;
mod rules;

extern crate clap;
use std::panic::set_hook;
use std::{net::TcpListener, sync::Arc, time::UNIX_EPOCH};

use crate::{australia::mappage::DefaultTuiMap, rules::Australia};
use async_recursion::async_recursion;
use australia::ScoreList;
use australia::showpage::ShowPage;
use australia::{mainpage::DefaultMainPage, mappage, Message};
use clap::{command, Parser, ValueEnum};
use log::{error, info, warn};
use rules::Scoring;
use rules::{cards::AustraliaCard, AustraliaPlayer, Event};
use server::engine;
use std::fs::File;
use std::io::Write;
use tokio::sync::broadcast::error::SendError;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf, ReadHalf, WriteHalf},
        TcpStream,
    },
    sync::broadcast::{self, Receiver, Sender},
    time::Instant,
};
use tui::tui::popup::info::Info;
use tui::tui::popup::select::Select;
use tui::{
    maps::boomerang_australia::Map,
    tui::{Tui, TuiMonitor},
    ui::Ui,
};

#[derive(Debug, Clone, Copy, ValueEnum)]
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
    #[arg(short = 'i', default_value = "0")]
    id: usize,
}

type TuiDefaults = Tui<
    DefaultMainPage<AustraliaCard, AustraliaPlayer>,
    mappage::DefaultTuiMap<Map, ScoreList>,
    ShowPage<AustraliaCard, AustraliaPlayer>,
    Info,
    Select,
>;
#[async_recursion]
async fn read_event(mut read_part: OwnedReadHalf, channel: broadcast::Sender<Event>) {
    loop {
        info!("Waiting for events");
        let mut buff = vec![0; 2048];
        let recv = read_part.read(&mut buff).await;
        match recv {
            Ok(_) => {}
            Err(_) => {
                continue;
            }
        }
        while let Some(0) = buff.last() {
            buff.pop();
        }

        let recv = String::from_utf8_lossy(&buff).to_string();
        info!("Server sent {:?}", recv);
        channel
            .send(match serde_json::from_str::<Event>(recv.as_str()) {
                Ok(val) => {
                    info!("returning {:?}", val);
                    val
                }
                _ => continue,
            })
            .unwrap();
    }
}

async fn send_event(write_part: &mut OwnedWriteHalf, event: Event) {
    let to_send: Vec<u8> = event.into();
    write_part.write_all(&to_send).await.unwrap();
}

async fn manage_event(
    writer: tokio::sync::broadcast::Sender<Message>,
    mut feedback_reader: tokio::sync::broadcast::Receiver<Message>,
    mut reader: Receiver<Event>,
    mut write_part: OwnedWriteHalf,
) {
    info!("Monitoring TCP");

    loop {
        let event: Event = match reader.recv().await {
            Ok(event) => event,
            _ => continue,
        };
        info!("Server sent {:?}", event);

        let to_send: Event = match event {
            Event::ReadyCheck => {
                writer.send(Message::ReadyCheck).unwrap();
                let mut ret = Event::Accept;
                loop {
                    warn!("Waiting for message from frontend");
                    match feedback_reader.recv().await {
                        Ok(Message::Ready) => {
                            info!("Message received from frontend");
                            ret = Event::Accept;
                            break;
                        }
                        Ok(Message::NotReady) => {
                            info!("Message received from frontend");
                            ret = Event::Deny;
                            break;
                        }
                        Err(_) => {
                            writer.send(Message::Exit).unwrap();
                            return;
                        }
                        _ => {}
                    }
                }
                ret
            }
            Event::Deal(card) => {
                writer.send(Message::Deal(card)).unwrap();
                Event::Accept
            }
            Event::UnexpectedMessage => continue,
            Event::DiscardRequest => {
                writer.send(Message::DiscardQuery).unwrap();
                let mut ret = 0;
                loop {
                    warn!("Waiting for message from frontend");
                    match feedback_reader.recv().await {
                        Ok(Message::Discard(_, idx)) => {
                            info!("Message received from frontend");
                            ret = idx;
                            break;
                        }
                        _ => {
                            writer.send(Message::Exit).unwrap();
                            return;
                        }
                    }
                }
                Event::Discard(ret)
            }
            Event::ShowRequest => {
                writer.send(Message::ShowQuery).unwrap();
                let mut ret = 0;
                loop {
                    warn!("Waiting for message from frontend");
                    match feedback_reader.recv().await {
                        Ok(Message::Show(_, idx)) => {
                            info!("Message received from frontend");
                            ret = idx;
                            break;
                        }
                        _ => return,
                    }
                }
                Event::Show(ret)
            }
            Event::ShowPile(idx, cards, visited) => {
                info!("Server sent ShowPile({:?},{:?})", idx, cards);
                writer
                    .send(Message::ShowOtherHand(idx.into(), cards, visited))
                    .unwrap();
                continue;
            }
            Event::ReassignHand(new_hand) => {
                info!("Replacing hand with with {:?}", new_hand);
                writer.send(Message::ReassignHand(new_hand)).unwrap();
                info!("Replaced");
                Event::Accept
            }
            Event::WaitingForPlayers => {
                writer.send(Message::WaitingForPlayers).unwrap();
                continue;
            }
            Event::Sync(player) => {
                loop {
                    match writer.send(Message::Sync(player.clone())) {
                        Ok(_) => break,
                        Err(_) => {
                            error!(
                                "Frontend managed must have crashed silently the channel is closed"
                            );
                            return;
                        }
                    }
                }
                loop {
                    warn!("Waiting for message from frontend");
                    match feedback_reader.recv().await {
                        Ok(Message::Ok) => {
                            break;
                        }
                        _ => return,
                    }
                }
                Event::Accept
            }
            Event::ScoreActivityQuery(options) => {
                writer.send(Message::ScoreActivityQuery(options)).unwrap();
                let mut ret = None;
                loop {
                    warn!("Waiting for message from frontend");
                    match feedback_reader.recv().await {
                        Ok(Message::ScoreActivity(x)) => {
                            info!("Message received from frontend");
                            ret = x;
                            break;
                        }
                        _ => {
                            continue;
                        }
                        Err(_) => {
                            writer.send(Message::Exit).unwrap();
                            return;
                        }
                        _ => {}
                    }
                }
                Event::ScoreActivity(ret)
            }
            Event::NewRound => {
                writer.send(Message::NewRound).unwrap();
                continue;
            }
            unexpected => {
                error!("Got unhandled message: {:?}", unexpected);
                continue;
            }
        }
        .into();

        send_event(&mut write_part, to_send).await;
    }
}

async fn player_main() {
    let (writer, reader) = tokio::sync::broadcast::channel::<Message>(32);
    let (feedback_writer, feedback_reader) = tokio::sync::broadcast::channel::<Message>(32);

    let join_handle = {
        let mainpage = DefaultMainPage::new();
        let mappage = DefaultTuiMap::new();
        let ui = Arc::new(TuiDefaults::init(mainpage, mappage));
        TuiDefaults::subscribe(ui.clone(), reader, feedback_writer);
        let ui_ref_clone = ui.clone();
        tokio::spawn(async move {
            TuiDefaults::start(ui_ref_clone).await;
        })
    };

    let stream = match TcpStream::connect("127.0.0.1:2047").await {
        Ok(val) => val,
        Err(e) => {
            println!("{:?}", e);
            panic!();
        }
    };
    let (read_part, mut write_part) = stream.into_split();
    let (broadcast_writer, broadcast_receiver) = broadcast::channel(32);
    let handle = tokio::spawn(async move { read_event(read_part, broadcast_writer).await });
    let handle = tokio::spawn(async move {
        manage_event(writer, feedback_reader, broadcast_receiver, write_part).await
    });
    info!("Started player");
    join_handle.await.unwrap();
    handle.await.unwrap();
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
    let log_file = File::create(format!("{:?}_{:?}_boomerang.log", args.mode, args.id))
        .expect("Failed to create log file");

    let started = Instant::now();

    env_logger::builder()
        .format(move |buf, record| {
            let now = Instant::now();
            let since_start = now.duration_since(started.clone()).as_secs();
            writeln!(
                buf,
                "[T: {} Log level : {:.6}] [{}] {} [{}:{}]",
                since_start,
                record.level(),
                record.args(),
                record.target(),
                record.file().unwrap_or("unknown file"),
                record.line().unwrap_or(0)
            )
        })
        .write_style(env_logger::WriteStyle::Never)
        .filter(None, log::LevelFilter::Info)
        .target(env_logger::Target::Pipe(Box::new(log_file)))
        .init();

    info!("App started with args : -m {:?}", args.mode);

    set_hook(Box::new(|panic_info| {
        let location = panic_info.location();
        let message = panic_info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| *s)
            .unwrap_or("Panic occurred");

        // Log the panic message and location.
        error!(target: "my_panic_handler", "Panic occurred: {} at {:?}", message, location);
    }));

    match args.mode {
        Mode::Server => server_main().await,
        Mode::Client => player_main().await,
    }
}
