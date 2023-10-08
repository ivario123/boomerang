use crate::ui::{Card, Hand, UiElement};

trait CardBuilder: UiElement {
    fn default() -> Self;
    fn set_name(self, name: &str) -> Self;
}
pub struct TuiCard {
    name: String,
}

pub struct TuiHand {
    cards: Vec<TuiCard>,
}

impl CardBuilder for TuiCard {
    fn default() -> Self {
        Self {
            name: "".to_owned(),
        }
    }

    fn set_name(mut self, name: &str) -> Self {
        self.name = name.to_owned();
        self
    }
}

impl UiElement for TuiCard {
    fn new() -> Self {
        Self::default()
    }
}

impl Card for TuiCard {
    fn get_name(&self) -> &str {
        &self.name
    }
}

impl UiElement for TuiHand {
    fn new() -> Self {
        Self {
            cards: vec![
                TuiCard::default().set_name("card 1"),
                TuiCard::default().set_name("card 2"),
                TuiCard::default().set_name("card 3"),
                TuiCard::default().set_name("card 4"),
                TuiCard::default().set_name("card 5"),
            ],
        }
    }
}
impl Hand<TuiCard> for TuiHand {
    fn get<const COUNT: usize>(&self, start: usize) -> (&[TuiCard], (usize, usize)) {
        match start > self.cards.len() - 1 {
            true => self.cards.len() - 1,
            false => start,
        };
        let end = match (start + COUNT) > self.cards.len() {
            false => start + COUNT,
            true => self.cards.len(),
        };
        (&self.cards[start..end], (end, self.cards.len()))
    }
    fn count(&self) -> usize {
        self.cards.len()
    }
}
