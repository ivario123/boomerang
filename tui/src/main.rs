use std::sync::Arc;
use tokio::sync::Mutex;
use tui::default::*;
use tui::Tui;

use tui::maps::boomerang_australia::*;
use crate::ui::Ui;

pub mod maps;
pub mod tui;
pub mod ui;

type TuiDefaults = Tui<DefaultMainPage<TuiHand>, tui::default::mappage::DefaultTuiMap<Map>>;
#[tokio::main]
async fn main() {
    let mainpage = DefaultMainPage::new();
    let mappage = DefaultTuiMap::new();

    let ui: Mutex<Box<TuiDefaults>> = TuiDefaults::init(mainpage, mappage);
    let _ = tokio::spawn(async {
        TuiDefaults::start(Arc::new(ui)).await;
    })
    .await;
    println!("Shutting down");
}
