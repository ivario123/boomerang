use ::tui::tui::{
    popup::{info::Info, select::Select},
    Tui,
};

use self::{
    rules::{cards::AustraliaCard, AustraliaPlayer},
    tui::{
        map::australia::Map,
        pages::{
            main_page::DefaultMainPage, map_page::DefaultTuiMap, score_popup::Score,
            show_page::ShowPage,
        },
        ScoreList,
    },
};

pub mod player;
pub mod protocol;
pub mod rules;
pub mod tui;

pub type TuiDefaults = Tui<
    DefaultMainPage<AustraliaCard, AustraliaPlayer>,
    DefaultTuiMap<Map, ScoreList>,
    ShowPage<AustraliaCard, AustraliaPlayer>,
    Info,
    Select,
    Score,
>;
