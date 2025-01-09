pub mod player;
use crate::{cards::air::steel_deck, evaluate::compare_hands, Error};
pub use player::{AirBios, Player};
use poise::serenity_prelude::{ComponentInteraction, UserId};
use std::sync::atomic::AtomicU8;

#[derive(Debug)]
pub struct AirPoker {
    // progresive blind
    pub blind: u8,
    // players in the game
    pub players: Vec<Player>,
    // bet round timeout
    pub bet_timeout: Option<AtomicU8>,
    // select card round timeout
    pub select_card_timeout: Option<AtomicU8>,
}

impl AirPoker {
    pub fn new(a: Player, b: Player) -> Self {
        Self {
            blind: 1,
            players: vec![a, b],
            bet_timeout: None,
            select_card_timeout: None,
        }
    }

    // delete select card timeout
    pub fn delete_select_card_timeout(&mut self) {
        self.select_card_timeout = None;
    }

    // 15 seconds to bet
    // used to set or reset the bet timeout in 15 seconds
    pub fn set_bet_timeout(&mut self) {
        self.bet_timeout = Some(AtomicU8::new(15));
    }

    // set select card timeout 15 if it doesn't exist
    pub fn set_select_card_timeout(&mut self) {
        self.select_card_timeout
            .get_or_insert_with(|| AtomicU8::new(15));
    }

    pub fn is_select_card_timeout(&self) -> bool {
        use std::sync::atomic::Ordering;
        self.select_card_timeout
            .as_ref()
            .map(|timeout| timeout.load(Ordering::Relaxed) == 0)
            .unwrap_or(false)
    }

    pub fn decrement_select_card_timeout(&self) {
        use std::sync::atomic::Ordering;
        if let Some(timeout) = self.select_card_timeout.as_ref() {
            timeout.fetch_sub(1, Ordering::Relaxed);
        }
    }

    // delete bet timeout
    // used to delete the bet timeout
    pub fn delete_bet_timeout(&mut self) {
        self.bet_timeout = None;
    }

    // check if bet timeout equals 0
    pub fn is_bet_timeout(&self) -> bool {
        use std::sync::atomic::Ordering;

        self.bet_timeout
            .as_ref()
            .map(|timeout| timeout.load(Ordering::Relaxed) == 0)
            .unwrap_or(false)
    }

    // decrement bet timeout if it exists
    pub fn decrement_bet_timeout(&self) {
        use std::sync::atomic::Ordering;
        if let Some(timeout) = self.bet_timeout.as_ref() {
            timeout.fetch_sub(1, Ordering::Relaxed);
        }
    }

    pub fn deal_cards(&mut self) {
        let mut deck = steel_deck();

        for player in self.players.iter_mut() {
            let hand = deck.drain(0..5).collect();
            player.hand = hand;
        }
    }

    pub fn compare_hands(&mut self) -> &Player {
        use std::cmp::Ordering;

        let a = self.players.first().unwrap();
        let b = self.players.last().unwrap();

        let a_hand = a.selected_card.clone().unwrap().hand;
        let b_hand = b.selected_card.clone().unwrap().hand;

        match compare_hands(&a_hand, &b_hand) {
            Ordering::Less => b,
            Ordering::Greater => a,
            Ordering::Equal => a,
        }
    }

    // reset players selected card
    pub fn reset_selected_cards(&mut self) {
        for player in self.players.iter_mut() {
            player.reset_selected_card();
        }
    }

    // get player by id
    pub fn get_player(&self, id: UserId) -> Result<&Player, Error> {
        self.players
            .iter()
            .find(|player| player.id == id)
            .ok_or_else(|| "Player not found".into())
    }

    // get mut player by id
    pub fn get_mut_player(&mut self, id: UserId) -> Result<&mut Player, Error> {
        self.players
            .iter_mut()
            .find(|player| player.id == id)
            .ok_or_else(|| "Player not found".into())
    }

    // check if players hands are empty
    pub fn empty_hands(&self) -> bool {
        self.players.iter().all(|player| player.hand.is_empty())
    }

    // check if all players have selected a card
    pub fn all_selected(&self) -> bool {
        self.players
            .iter()
            .all(|player| player.selected_card.is_some())
    }

    // set players bet to the value of the blind
    pub fn set_players_blind(&mut self) {
        for player in self.players.iter_mut() {
            if player.air_bios.len() < self.blind as usize {
                player.bet = player.air_bios.len() as u8;
            } else {
                player.bet = self.blind;
            }
        }
    }

    // find the one that has not selected a card
    // return player mut
    pub fn find_not_selected(&mut self) -> Result<&mut Player, Error> {
        self.players
            .iter_mut()
            .find(|player| player.selected_card.is_none())
            .ok_or_else(|| "Player not found".into())
    }
}
