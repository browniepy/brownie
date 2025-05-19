use crate::cards::poker::{BjValue, Card};
use inflector::Inflector;
use poise::serenity_prelude::{User, UserId};

#[derive(Clone, Debug, PartialEq)]
pub enum State {
    Stand,
    Bust,
    None,
    Blackjack,
}

#[derive(Clone, Debug)]
pub struct Player {
    pub id: UserId,
    pub name: String,
    pub bet: i64,
    pub hand: Vec<Card>,
    pub state: State,
}

#[derive(Default)]
pub struct Dealer {
    pub hand: Vec<Card>,
}

impl Dealer {
    pub fn hand_value(&self, hidden: bool) -> i32 {
        if hidden {
            self.hand[0].value()
        } else {
            let mut total = 0;
            let mut aces = 0;

            for card in self.hand.iter() {
                total += card.value();

                if card.is_ace() {
                    aces += 1;
                }
            }

            while total > 21 && aces > 0 {
                total -= 10;
                aces -= 1;
            }

            total
        }
    }

    pub fn is_bust(&self) -> bool {
        self.hand_value(false) > 21
    }

    pub fn dbg_hand(&self, hidden: bool) -> String {
        let mut hand = self
            .hand
            .iter()
            .map(|card| format!("{:?}", card))
            .collect::<Vec<String>>();

        if hidden {
            hand[1] = "Card(?)".to_string();
        }

        format!("{:?}", hand)
    }
}

impl Player {
    pub fn new(user: User, bet: i64) -> Self {
        let name = user.display_name().to_title_case();
        let id = user.id;

        Self {
            id,
            name,
            bet,
            hand: Vec::new(),
            state: State::None,
        }
    }

    pub fn dbg_hand(&self) -> String {
        let hand = self
            .hand
            .iter()
            .map(|card| format!("{:?}", card))
            .collect::<Vec<String>>();

        format!("{:?}", hand)
    }

    pub fn is_stand(&self) -> bool {
        self.state == State::Stand
    }

    pub fn is_bust(&self) -> bool {
        self.state == State::Bust
    }

    pub fn is_blackjack(&self) -> bool {
        self.state == State::Blackjack
    }

    pub fn is_blackjack_and_set(&mut self) -> bool {
        if self.hand.len() == 2 && self.hand_value() == 21 {
            self.state = State::Blackjack;
            true
        } else {
            false
        }
    }

    // can hit if not stand, blackjack or bust
    pub fn can_hit(&mut self) -> bool {
        !self.is_stand() && !self.is_blackjack_and_set() && !self.is_bust()
    }

    // cant stand if not blackjack or bust
    pub fn can_stand(&mut self) -> bool {
        !self.is_blackjack_and_set() && !self.is_bust()
    }

    pub fn hand_value(&self) -> i32 {
        let mut total = 0;
        let mut aces = 0;

        for card in self.hand.iter() {
            total += card.value();

            if card.is_ace() {
                aces += 1;
            }
        }

        while total > 21 && aces > 0 {
            total -= 10;
            aces -= 1;
        }

        total
    }
}
