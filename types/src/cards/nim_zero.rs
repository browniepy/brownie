#[derive(Clone, Debug)]
pub enum Card {
    Zero,
    One,
    Two,
    Three,
}

impl Card {
    pub fn standart_deck() -> Vec<Self> {
        use rand::seq::SliceRandom;

        let mut deck = Vec::new();

        for _ in 0..10 {
            deck.extend([Self::Zero, Self::One, Self::Two, Self::Three]);
        }

        deck.shuffle(&mut rand::thread_rng());
        deck
    }

    pub fn value(&self) -> i32 {
        match *self {
            Self::Zero => 0,
            Self::One => 1,
            Self::Two => 2,
            Self::Three => 3,
        }
    }

    pub fn name(&self) -> String {
        match *self {
            Self::Zero => String::from("zero"),
            Self::One => String::from("one"),
            Self::Two => String::from("two"),
            Self::Three => String::from("three"),
        }
    }
}
