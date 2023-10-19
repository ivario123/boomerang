use ::tui::tui::{Tui, popup::{info::Info, select::Select}};

use self::{tui::{pages::{main_page::DefaultMainPage, map_page::DefaultTuiMap, show_page::ShowPage}, map::australia::Map, ScoreList}, rules::{cards::AustraliaCard, AustraliaPlayer}};


pub mod tui;
pub mod protocol;
pub mod rules;
pub mod player;




pub type TuiDefaults = Tui<
    DefaultMainPage<AustraliaCard, AustraliaPlayer>,
    DefaultTuiMap<Map, ScoreList>,
    ShowPage<AustraliaCard, AustraliaPlayer>,
    Info,
    Select,
>;