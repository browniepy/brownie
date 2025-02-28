use super::poker::{Card, PokerValue};

#[derive(Debug, Clone)]
pub struct SteelCard {
    // representative hand for the value
    pub hand: Vec<Card>,
    pub witch_hand: Vec<Card>,
}

impl SteelCard {
    pub fn new(hand: Vec<Card>) -> Self {
        Self {
            hand,
            witch_hand: Vec::new(),
        }
    }

    pub fn value(&self) -> u8 {
        let sum: u8 = self.hand.iter().map(|c| c.value()).sum();
        sum
    }

    pub fn set_witch_hand(&mut self, hand: Vec<Card>) {
        self.witch_hand = hand;
    }

    pub fn witch_value(&self) -> u8 {
        let sum: u8 = self.witch_hand.iter().map(|c| c.value()).sum();
        sum
    }
}

// create a standard deck of poker
// then 10 steel cards with 5 poker cards each
pub fn steel_deck() -> Vec<SteelCard> {
    let mut deck = Card::standart_deck();

    let mut steel_deck = Vec::new();

    for _ in 0..10 {
        let hand = deck.drain(0..5).collect();
        steel_deck.push(SteelCard::new(hand));
    }

    steel_deck
}
