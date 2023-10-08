use ratatui::{
    prelude::{Backend, Rect},
    style::{Style, Stylize},
    symbols::Marker,
    widgets::{canvas::Canvas, Block, Borders, Tabs},
    Frame,
};

use crate::{
    maps::{self, sites::Region, Map},
    ui::{self, Card, Hand, UiElement},
};

use self::card::TuiCard;

use super::TuiPage;

pub mod card;
pub mod mainpage;
pub mod mappage;
pub mod paginate;
