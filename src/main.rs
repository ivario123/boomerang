//! Boomerang
//! 
//! Implements both a boomerang client and a boomerang server.
//! The only rule set provided is the BoomerangAustralia rule set.
//! 
//! ## Crates
//! This crate provides all of the needed tools to run a boomerang australia game.
//! But the other crates in this repository are totally generic to what rule set is
//! being used, in fact [`server`] is generic to what type of turn based game is being played.
//! 
//! The [`tui`] however is implemented for something similar to the boomerang australia game. 


use std::{fs::File, io::Write, panic::set_hook, sync::Arc};

use clap::{Parser, ValueEnum};
use log::{error, info};
use server::engine;
use tokio::{net::TcpStream, sync::broadcast, time::Instant};
use tui::{tui::TuiMonitor, ui::Ui};

use crate::australia::{
    player::{manage_event, read_event},
    protocol::Message,
    rules::Australia,
    tui::pages::{main_page::MainPage, map_page::DefaultTuiMap},
    TuiDefaults,
};
mod australia;

/// The modes that the app can run in
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Mode {
    Server,
    Client,
}

#[derive(Parser)]
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

async fn player_main() {
    let (writer, reader) = tokio::sync::broadcast::channel::<Message>(32);
    let (feedback_writer, feedback_reader) = tokio::sync::broadcast::channel::<Message>(32);

    let join_handle = {
        let main_page = MainPage::new();
        let map_page = DefaultTuiMap::new();
        let ui = Arc::new(TuiDefaults::init(main_page, map_page));
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
    let (read_part, write_part) = stream.into_split();
    let (broadcast_writer, broadcast_receiver) = broadcast::channel(32);
    let _handle = tokio::spawn(async move { read_event(read_part, broadcast_writer).await });
    let handle = tokio::spawn(async move {
        manage_event(writer, feedback_reader, broadcast_receiver, write_part).await
    });
    info!("Started player");
    join_handle.await.unwrap();
    handle.await.unwrap();
}

async fn server_main() {
    println!("Running as server");
    let listener = match std::net::TcpListener::bind("127.0.0.1:2047") {
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
