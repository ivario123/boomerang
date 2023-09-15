use super::card::Card;

pub struct Hand {
    cards: Vec<Box<dyn Card>>,
}
