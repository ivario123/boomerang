use ratatui::{
    prelude::{Backend, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tui::{
    tui::{controls::EventApi, TuiPage},
    ui::{Card as _, Hand},
};

use self::pages::main_page::CardArea;

use super::rules::{
    cards::{AustraliaCard, Card},
    scoring::Scoring,
    AustraliaPlayer,
};

pub mod map;
pub mod mediator;
pub mod pages;

pub struct ScoreList(Vec<Scoring>);

impl EventApi for ScoreList {
    fn handle_input(&mut self, _control: tui::tui::controls::Controls) {
        todo!()
    }
}
impl Default for ScoreList {
    fn default() -> Self {
        Self(Vec::new())
    }
}
impl ScoreList {
    // Helper function to format elements as "{el1} + {el2} + .... + {last} = {sum}"
    fn format_elements(&self, elements: Vec<usize>) -> String {
        // Check if the elements vector is empty
        if elements.is_empty() {
            return String::from("{}");
        }

        // If there's only one element, format it without brackets
        if elements.len() == 1 {
            return elements[0].to_string();
        }

        // Create a vector to hold the formatted elements
        let mut formatted_elements = Vec::new();

        // Format elements as "{el1} + {el2} + .... + {last} = {sum}"
        for el in &elements {
            formatted_elements.push(format!("{:?}", el));
        }
        let sum: usize = elements.iter().sum();
        let ret = vec![formatted_elements.join(" + "), format!("{:?}", sum)];
        ret.join("=")
    }
}
impl TuiPage for ScoreList {
    fn draw<B: Backend>(&mut self, frame: &mut Frame<B>, block: Rect) {
        let score_area = Block::default()
            .title("Score")
            .borders(Borders::all())
            .style(Style::default());
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage((100 / 5) as u16); 5].as_ref())
            .split(block)
            .to_vec();

        let mut paragraphs = Vec::new();

        let mut throw_catch = Vec::new();
        let mut tourist_sites = Vec::new();
        let mut collections = Vec::new();
        let mut animals = Vec::new();
        let mut activity = Vec::new();
        for score in &self.0 {
            throw_catch.push(score.throw_catch());
            tourist_sites.push(score.tourist_sites());
            collections.push(score.collections());
            animals.push(score.animals());
            activity.push(score.activity());
        }

        paragraphs.push(Paragraph::new(format!("Throw Catch : {:?}", throw_catch)));
        paragraphs.push(Paragraph::new(format!(
            "tourist_sites : {:?}",
            self.format_elements(tourist_sites)
        )));
        paragraphs.push(Paragraph::new(format!(
            "Collections : {:?}",
            self.format_elements(collections)
        )));
        paragraphs.push(Paragraph::new(format!(
            "Animals : {:?}",
            self.format_elements(animals)
        )));
        paragraphs.push(Paragraph::new(format!(
            "Activity : {:?}",
            self.format_elements(activity)
        )));
        frame.render_widget(score_area, block);
        for (block, paragraph) in layout.iter().zip(paragraphs) {
            frame.render_widget(paragraph, *block);
        }
    }

    fn set_title(&mut self, _title: String) {}

    fn get_title(&self) -> &str {
        "Score"
    }
}

impl CardArea<AustraliaCard> for AustraliaPlayer
where
    Self: Hand<AustraliaCard>,
{
    fn increment(&mut self) {
        let count = self.hand_size();
        let ptr = self.card_ptr();
        if count > 0 && *ptr < count - 1 {
            *ptr += 1;
        }
    }

    fn decrement(&mut self) {
        let ptr = self.card_ptr();
        if *ptr > 0 {
            *ptr -= 1;
        }
    }

    fn card_ptr(&mut self) -> &mut usize {
        self.card_ptr()
    }

    fn draw<B: Backend>(&mut self, frame: &mut Frame<B>, block: Rect, title: &str, border: Color) {
        let display_ptr = *self.card_ptr();
        let (cards, (last, count)) = self.get_cards::<4>(display_ptr);

        let card_area = Block::default()
            .title(format!(
                "{} (showing {} - {}/{})",
                title,
                display_ptr + 1,
                last,
                count
            ))
            .borders(Borders::all())
            .style(Style::default().fg(border));
        frame.render_widget(card_area, block);

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Percentage((100 / 4) as u16); 4].as_ref())
            .split(block);
        let num_cards = cards.len();

        for (idx, (area, card)) in layout.iter().zip(cards).enumerate() {
            let rect = Block::default()
                .title(card.get_name())
                .borders(Borders::all())
                .border_style(
                    Style::default().fg(match (idx + last - num_cards) == display_ptr {
                        true => Color::Yellow,
                        false => Color::Gray,
                    }),
                );
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Percentage((100 / 5) as u16); 5].as_ref())
                .split(*area)
                .to_vec();

            let mut paragraphs = Vec::new();
            paragraphs.push(Paragraph::new(format!(
                "Site : {:?} | # : {:?}",
                card.to_char(),
                Card::number(card)
            )));
            paragraphs.push(Paragraph::new(format!("Region : {:?}", card.region())));
            if let Some(collection) = card.collection() {
                paragraphs.push(Paragraph::new(format!("Collection : {:?}", collection)));
            }
            if let Some(animal) = card.animal() {
                paragraphs.push(Paragraph::new(format!("Animal : {:?}", animal)));
            }
            if let Some(activity) = card.activity() {
                paragraphs.push(Paragraph::new(format!("Activity : {:?}", activity)));
            }

            frame.render_widget(rect, *area);
            for (area, par) in layout.iter().zip(paragraphs) {
                frame.render_widget(par, *area);
            }
        }
    }
}
