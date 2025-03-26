use crate::cards::nim_zero::Card;
use poise::serenity_prelude::{Mentionable, User, UserId};

#[derive(Clone)]
pub struct Player {
    pub id: UserId,
    pub name: String,
    pub hand: Vec<Card>,
    pub wins: usize,
    pub bot: bool,
}

impl Player {
    pub fn new(user: Option<&User>, bot: bool) -> Self {
        let name = if bot {
            String::from("Machine")
        } else {
            user.unwrap().mention().to_string()
        };

        Self {
            id: user.unwrap().id,
            name,
            hand: Vec::new(),
            wins: 0,
            bot,
        }
    }

    pub fn one_card_left(&self) -> bool {
        self.hand.iter().filter(|c| !c.disabled).count() == 1
    }

    pub fn is_bot(&self) -> bool {
        self.bot
    }
}
