use inflector::Inflector;
use poise::serenity_prelude::{User, UserId};

use crate::{airpoker::AirPoker, cards::poker::Card};

pub enum Pains {}

pub struct WitchPlayer {
    pub id: UserId,
    pub name: String,
    pub ally: UserId,
    pub selected_hand: Vec<Card>,
    pub suffered_pains: Vec<Pains>,
}

pub struct WitchPain {
    pub players: Vec<WitchPlayer>,
    pub deck: Vec<Card>,
    pub used_cards: Vec<Card>,
    pub pains: Vec<Pains>,
}

impl Pains {
    pub fn new() -> Vec<Self> {
        Vec::new()
    }
}

impl WitchPlayer {
    pub fn new(id: UserId, name: String, ally: UserId) -> Self {
        Self {
            id,
            name,
            ally,
            selected_hand: Vec::new(),
            suffered_pains: Vec::new(),
        }
    }
}

impl WitchPain {
    pub fn new(a: &User, a_ally: UserId, b: &User, b_ally: UserId) -> Self {
        let players = vec![
            WitchPlayer::new(a.id, a.display_name().to_title_case(), a_ally),
            WitchPlayer::new(b.id, b.display_name().to_title_case(), b_ally),
        ];

        Self {
            players,
            deck: Card::standart_deck(),
            used_cards: Vec::new(),
            pains: Pains::new(),
        }
    }
}
