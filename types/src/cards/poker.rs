use rand::seq::SliceRandom;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Card {
    Two(Suit),
    Three(Suit),
    Four(Suit),
    Five(Suit),
    Six(Suit),
    Seven(Suit),
    Eight(Suit),
    Nine(Suit),
    Ten(Suit),
    Jack(Suit),
    Queen(Suit),
    King(Suit),
    Ace(Suit),
    Joker(Suit),
}

pub trait PokerValue {
    fn value(&self) -> u8;
}

pub trait BjValue {
    fn value(&self) -> i32;
}

impl PokerValue for Card {
    fn value(&self) -> u8 {
        match *self {
            Self::Two(_) => 2,
            Self::Three(_) => 3,
            Self::Four(_) => 4,
            Self::Five(_) => 5,
            Self::Six(_) => 6,
            Self::Seven(_) => 7,
            Self::Eight(_) => 8,
            Self::Nine(_) => 9,
            Self::Ten(_) => 10,
            Self::Jack(_) => 11,
            Self::Queen(_) => 12,
            Self::King(_) => 13,
            Self::Ace(_) => 1,
            Self::Joker(_) => 0,
        }
    }
}

impl BjValue for Card {
    fn value(&self) -> i32 {
        match *self {
            Card::Two(_) => 2,
            Card::Three(_) => 3,
            Card::Four(_) => 4,
            Card::Five(_) => 5,
            Card::Six(_) => 6,
            Card::Seven(_) => 7,
            Card::Eight(_) => 8,
            Card::Nine(_) => 9,
            Card::Ten(_) | Card::Queen(_) | Card::King(_) | Card::Jack(_) => 10,
            Card::Ace(_) => 11,
            Self::Joker(_) => 0,
        }
    }
}

impl Card {
    pub fn is_ace(&self) -> bool {
        matches!(*self, Card::Ace(_))
    }

    pub fn is_joker(&self) -> bool {
        matches!(*self, Card::Joker(_))
    }

    pub fn standart_deck() -> Vec<Self> {
        let mut deck = Vec::new();

        for pale in [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades] {
            deck.extend([
                Self::Two(pale),
                Self::Three(pale),
                Self::Four(pale),
                Self::Five(pale),
                Self::Six(pale),
                Self::Seven(pale),
                Self::Eight(pale),
                Self::Nine(pale),
                Self::Ten(pale),
                Self::Queen(pale),
                Self::King(pale),
                Self::Ace(pale),
                Self::Jack(pale),
            ]);
        }

        deck.shuffle(&mut rand::thread_rng());
        deck
    }

    pub fn black_deck() -> Vec<Self> {
        let mut deck = Vec::new();

        for pale in [Suit::Clubs, Suit::Spades] {
            deck.extend([
                Self::Two(pale),
                Self::Three(pale),
                Self::Four(pale),
                Self::Five(pale),
                Self::Six(pale),
                Self::Seven(pale),
                Self::Eight(pale),
                Self::Nine(pale),
                Self::Ten(pale),
                Self::Queen(pale),
                Self::King(pale),
                Self::Ace(pale),
                Self::Jack(pale),
            ]);
        }

        deck.push(Self::Joker(Suit::Spades));

        deck.shuffle(&mut rand::thread_rng());
        deck
    }

    pub fn key_name(&self) -> String {
        match *self {
            Self::Two(_) => String::from("two"),
            Self::Three(_) => String::from("three"),
            Self::Four(_) => String::from("four"),
            Self::Five(_) => String::from("five"),
            Card::Six(_) => String::from("six"),
            Self::Seven(_) => String::from("seven"),
            Self::Eight(_) => String::from("eight"),
            Self::Nine(_) => String::from("nine"),
            Self::Ten(_) => String::from("ten"),
            Self::Jack(_) => String::from("jack"),
            Self::Queen(_) => String::from("queen"),
            Self::King(_) => String::from("king"),
            Self::Ace(_) => String::from("ace"),
            Self::Joker(_) => String::from("joker"),
        }
    }

    pub fn suit(&self) -> Suit {
        match *self {
            Self::Joker(s)
            | Self::Two(s)
            | Self::Three(s)
            | Self::Four(s)
            | Self::Five(s)
            | Self::Six(s)
            | Self::Seven(s)
            | Self::Eight(s)
            | Self::Nine(s)
            | Self::Ten(s)
            | Self::Jack(s)
            | Self::Queen(s)
            | Self::King(s)
            | Self::Ace(s) => s,
        }
    }
}
