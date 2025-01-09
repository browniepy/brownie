use std::{error::Error, string::ToString};
use strum::{Display, EnumString};

pub type ErrorT = Box<dyn Error + Send + Sync>;

#[derive(EnumString)]
pub enum Command {
    Work,
}

#[derive(Debug, EnumString, Display, Clone, Copy)]
#[strum(serialize_all = "snake_case")]
pub enum Pale {
    #[strum(to_string = "club")]
    Club,
    #[strum(to_string = "diamond")]
    Diamond,
    #[strum(to_string = "heart")]
    Heart,
    #[strum(to_string = "spade")]
    Spade,
}

impl Default for Pale {
    fn default() -> Self {
        Self::Club
    }
}

#[derive(Debug, EnumString, Clone, Copy)]
#[strum(serialize_all = "snake_case")]
pub enum Card {
    Two(Pale),
    Three(Pale),
    Four(Pale),
    Five(Pale),
    Six(Pale),
    Seven(Pale),
    Eight(Pale),
    Nine(Pale),
    Ten(Pale),
    Joker(Pale),
    Queen(Pale),
    King(Pale),
    Ace(Pale),
}

impl Card {
    pub fn value(&self) -> i32 {
        match *self {
            Card::Two(_) => 2,
            Card::Three(_) => 3,
            Card::Four(_) => 4,
            Card::Five(_) => 5,
            Card::Six(_) => 6,
            Card::Seven(_) => 7,
            Card::Eight(_) => 8,
            Card::Nine(_) => 9,
            Card::Ten(_) | Card::Queen(_) | Card::King(_) | Card::Joker(_) => 10,
            Card::Ace(_) => 11,
        }
    }

    pub fn name(&self) -> String {
        let value = match *self {
            Self::Two(_) => "2",
            Card::Three(_) => "3",
            Card::Four(_) => "4",
            Self::Five(_) => "5",
            Card::Six(_) => "6",
            Card::Seven(_) => "7",
            Card::Eight(_) => "8",
            Card::Nine(_) => "9",
            Card::Ten(_) => "10",
            Card::Joker(_) => "j",
            Card::Queen(_) => "q",
            Card::King(_) => "k",
            Card::Ace(_) => "a",
        };

        let pale = match *self {
            Card::Two(p)
            | Self::Three(p)
            | Self::Four(p)
            | Self::Five(p)
            | Card::Six(p)
            | Card::Seven(p)
            | Self::Eight(p)
            | Self::Nine(p)
            | Self::Ten(p)
            | Self::Ace(p)
            | Self::King(p)
            | Self::Queen(p)
            | Self::Joker(p) => match p {
                Pale::Club => p.to_string(),
                Pale::Diamond => p.to_string(),
                Pale::Heart => p.to_string(),
                Pale::Spade => p.to_string(),
            },
        };

        format!("{} {}", value, pale)
    }
}

/// Creates a standart deck
pub fn create_deck() -> Vec<Card> {
    use rand::seq::SliceRandom;

    let mut deck = Vec::new();

    for pale in [Pale::Club, Pale::Diamond, Pale::Heart, Pale::Spade] {
        deck.extend([
            Card::Two(pale),
            Card::Three(pale),
            Card::Four(pale),
            Card::Five(pale),
            Card::Six(pale),
            Card::Seven(pale),
            Card::Eight(pale),
            Card::Nine(pale),
            Card::Ten(pale),
            Card::Queen(pale),
            Card::King(pale),
            Card::Ace(pale),
            Card::Joker(pale),
        ]);
    }

    deck.shuffle(&mut rand::thread_rng());
    deck
}

/// calculates the total value of a deck
pub fn calculate_deck(deck: &[Card]) -> i32 {
    let mut total = 0;
    let mut num_ases = 0;

    for card in deck {
        total += card.value();

        if let Card::Ace(_) = card {
            num_ases += 1;
        }
    }

    while total > 21 && num_ases > 0 {
        total -= 10;
        num_ases -= 1;
    }

    total
}

/// shows a deck
pub fn show_deck(deck: &[Card], hide_first: bool) -> Vec<String> {
    let mut showed_deck = Vec::new();

    if hide_first {
        showed_deck.push(String::from("?"));

        for card in deck.iter().skip(1) {
            showed_deck.push(card.name());
        }
    } else {
        for card in deck {
            showed_deck.push(card.name());
        }
    }

    showed_deck
}
