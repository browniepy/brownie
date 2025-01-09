mod player;
use crate::Error;
use std::time::{Duration, Instant};

use crate::cards::poker::Card;
pub use player::Player;
use poise::serenity_prelude::{User, UserId};

pub struct Oldmaid {
    pub players: Vec<Player>,
    pub turn_timeout: Option<Duration>,
    // new message timeout
    pub message_timeout: Option<Instant>,
}

impl Oldmaid {
    pub fn new(author: &User) -> Self {
        Self {
            players: vec![Player::new(author)],
            turn_timeout: None,
            message_timeout: None,
        }
    }

    pub fn add_player(&mut self, user: &User) -> Result<(), Error> {
        if self.players.len() == 2 {
            return Err("Game is full".into());
        }

        self.players.push(Player::new(user));
        Ok(())
    }

    // amount of discarded pairs
    pub fn discarded_pairs(&self) -> usize {
        let sum: usize = self
            .players
            .iter()
            .map(|player| player.discarted_cards.len())
            .sum();
        sum / 2
    }

    // get player by id
    pub fn get_player_mut(&mut self, id: UserId) -> &mut Player {
        self.players.iter_mut().find(|p| p.id == id).unwrap()
    }

    // set new message timeout
    pub fn trigger_message_timeout(&mut self) {
        self.message_timeout = Some(Instant::now())
    }

    pub fn trigger_timeout(&mut self) {
        self.turn_timeout = Some(Duration::from_secs(500));
    }

    pub fn get_player(&self, id: UserId) -> &Player {
        self.players.iter().find(|p| p.id == id).unwrap()
    }

    pub fn deal_cards(&mut self) {
        let mut deck = Card::black_deck();

        let fp = self.players.first_mut().unwrap();
        fp.hand.extend(deck.drain(0..13));

        let sp = self.players.last_mut().unwrap();
        sp.hand.extend(deck.drain(0..));
    }

    pub fn next_turn(&mut self) {
        let player = self.players.remove(0);
        self.players.push(player);
    }

    pub fn get_mut_rival(&mut self) -> &mut Player {
        self.players.last_mut().unwrap()
    }

    pub fn get_actual(&self) -> &Player {
        self.players.first().unwrap()
    }

    pub fn get_rival(&self) -> &Player {
        self.players.last().unwrap()
    }

    pub fn cards_in_game(&self) -> usize {
        self.players.iter().map(|player| player.hand.len()).sum()
    }

    // discard pairs from players hands
    pub fn discard_pairs(&mut self) {
        for player in self.players.iter_mut() {
            player.discard_pairs();
        }
    }

    // loser
    pub fn get_player_with_oldmaid(&self) -> &Player {
        self.players
            .iter()
            .find(|player| player.hand.iter().any(|card| card.is_joker()))
            .unwrap()
    }

    // winner (player without cards)
    pub fn get_winner(&self) -> &Player {
        self.players
            .iter()
            .find(|player| player.hand.is_empty())
            .unwrap()
    }

    // reset confirmed card index of all players
    pub fn reset_confirmed_card_index(&mut self) {
        for player in self.players.iter_mut() {
            player.confirmed_card_index = None;
        }
    }
}
