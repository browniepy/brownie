use crate::cards::nim_zero::Card;
use inflector::Inflector;
use poise::serenity_prelude::{Mentionable, User, UserId};

pub struct Player {
    pub id: UserId,
    pub name: String,
    pub hand: Vec<Card>,
    pub wins: usize,
}

impl Player {
    pub fn new(user: &User) -> Self {
        let name = user.mention().to_string();

        Self {
            id: user.id,
            name,
            hand: Vec::new(),
            wins: 0,
        }
    }

    pub fn one_card_left(&self) -> bool {
        self.hand.iter().filter(|c| !c.disabled).count() == 1
    }
}
