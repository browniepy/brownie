mod player;
pub use player::Player;

use super::{cards::nim_zero::Card, Error};
use poise::serenity_prelude::{MessageId, UserId};

pub struct Nim {
    pub players: Vec<Player>,
    pub deck: Vec<Card>,
    pub table_cards: Vec<Card>,
    pub ephemeral: Option<MessageId>,
    pub bet: i32,
    last_played_card: Option<Card>,
}

impl Nim {
    pub fn new(player: Player, bet: i32) -> Self {
        Self {
            players: vec![player],
            deck: Card::standart_deck(),
            table_cards: Vec::new(),
            ephemeral: None,
            bet,
            last_played_card: None,
        }
    }

    pub fn add_player(&mut self, player: Player) -> Result<(), Error> {
        if self.players.len() == 2 {
            return Err("game is full".into());
        }

        self.players.push(player);
        Ok(())
    }

    pub fn table_value(&self) -> i32 {
        self.table_cards.iter().map(|card| card.value()).sum()
    }

    pub fn deal_cards(&mut self) {
        for player in self.players.iter_mut() {
            player.hand = self.deck.drain(0..5).collect();
        }
    }

    pub fn current_player(&self) -> &Player {
        self.players.first().unwrap()
    }

    pub fn mut_current_player(&mut self) -> &mut Player {
        self.players.first_mut().unwrap()
    }

    pub fn mut_rival_player(&mut self) -> &mut Player {
        self.players.last_mut().unwrap()
    }

    pub fn rival_player(&self) -> &Player {
        self.players.last().unwrap()
    }

    pub fn next_player(&mut self) {
        let player = self.players.remove(0);
        self.players.push(player);
    }

    pub fn hand_is_empty(&self) -> bool {
        self.current_player().hand.is_empty()
    }

    pub fn one_card_left(&self) -> bool {
        self.current_player().hand.len() == 1
    }

    pub fn check_hand(&mut self) {
        if self.hand_is_empty() {
            let card = self.deck.remove(0);
            self.mut_current_player().hand.push(card);
        }
    }

    pub fn play_card(&mut self, index: usize) {
        {
            let card = self.mut_current_player().hand.get_mut(index).unwrap();
            card.disabled = true;
        }

        let card = self.current_player().hand.get(index).unwrap().clone();
        self.table_cards.push(card.clone());
        self.last_played_card = Some(card);
    }

    pub fn play_unique_card(&mut self) {
        let index = self
            .current_player()
            .hand
            .iter()
            .position(|card| !card.disabled)
            .unwrap();

        self.play_card(index);
    }

    pub fn last_played_card(&self) -> &Card {
        self.last_played_card.as_ref().unwrap()
    }

    pub fn get_winner(&self) -> Option<&Player> {
        self.players.iter().find(|player| player.wins == 2)
    }

    pub fn get_loser(&self) -> Option<&Player> {
        self.players.iter().find(|player| player.wins <= 1)
    }

    pub fn get_player(&self, id: UserId) -> &Player {
        self.players.iter().find(|player| player.id == id).unwrap()
    }

    pub fn has_winner(&self) -> bool {
        self.get_winner().is_some()
    }
}
