use inflector::Inflector;
use poise::serenity_prelude::{User, UserId};

use crate::{translation::translate, Context};

#[derive(Clone, Debug)]
pub enum Card {
    Zero,
    One,
    Two,
    Three,
}

impl Card {
    pub fn std_deck() -> Vec<Self> {
        use rand::seq::SliceRandom;

        let mut deck = Vec::new();

        for _ in 0..10 {
            deck.extend([Self::Zero, Self::One, Self::Two, Self::Three]);
        }

        deck.shuffle(&mut rand::thread_rng());
        deck
    }

    pub fn value(&self) -> u8 {
        match *self {
            Self::Zero => 0,
            Self::One => 1,
            Self::Two => 2,
            Self::Three => 3,
        }
    }

    pub fn name(&self, ctx: Context<'_>) -> String {
        match *self {
            Self::Zero => translate!(ctx, "zero"),
            Self::One => translate!(ctx, "one"),
            Self::Two => translate!(ctx, "two"),
            Self::Three => translate!(ctx, "three"),
        }
    }
}

pub enum RoundState {
    Finish { loser: Player, winner: Player },
    Continue,
}

#[derive(Clone)]
pub struct Player {
    pub id: UserId,
    pub name: String,
    pub hand: Vec<Card>,
    pub wins: u8,
}

pub struct Nim {
    pub players: Vec<Player>,
    pub deck: Vec<Card>,
    pub table_cards: Vec<Card>,
    pub score: u8,
}

impl Player {
    pub fn new(user: User) -> Self {
        let name = user.display_name().to_title_case();

        // let name = format!("<@{}>", user.id);

        Self {
            id: user.id,
            name,
            hand: Vec::new(),
            wins: 0,
        }
    }
}

impl Nim {
    pub fn new(a: User, b: User) -> Self {
        Self {
            players: vec![Player::new(a), Player::new(b)],
            deck: Card::std_deck(),

            table_cards: Vec::new(),
            score: 0,
        }
    }

    pub fn deal_cards(&mut self) {
        for player in self.players.iter_mut() {
            player.hand = self.deck.drain(0..5).collect();
        }
    }

    pub fn next_player(&mut self) -> &Player {
        let player = self.players.remove(0);
        self.players.push(player);
        self.current_player()
    }

    pub fn mut_current_player(&mut self) -> &mut Player {
        self.players.first_mut().unwrap()
    }

    pub fn get_mut_rival(&mut self) -> &mut Player {
        self.players.last_mut().unwrap()
    }

    pub fn current_player(&self) -> &Player {
        self.players.first().unwrap()
    }

    pub fn put_card(&mut self, index: usize) -> Card {
        let curr_player = self.mut_current_player();
        let card = curr_player.hand.remove(index);

        self.score += card.value();
        self.table_cards.push(card.clone());
        card
    }

    pub fn reset_state(&mut self) {
        self.table_cards.clear();
        self.score = 0;
    }

    pub fn round_state(&mut self) -> RoundState {
        if self.score >= 9 {
            RoundState::Finish {
                loser: self.current_player().clone(),
                winner: self.next_player().clone(),
            }
        } else {
            RoundState::Continue
        }
    }

    pub fn game_winner(&self) -> Option<&Player> {
        self.players.iter().find(|player| player.wins == 2)
    }

    pub fn finish(&self) -> bool {
        self.players.iter().any(|player| player.wins == 2)
    }
}
