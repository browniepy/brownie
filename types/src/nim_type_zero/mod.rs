mod player;

use super::{cards::nim_zero::Card, Error};
pub use player::Player;
use poise::serenity_prelude::{MessageId, UserId};
use std::time::Duration;

#[derive(Clone)]
pub struct Nim {
    pub players: Vec<Player>,
    pub deck: Vec<Card>,
    pub table_cards: Vec<Card>,
    pub ephemeral: Option<MessageId>,
    pub bet: i64,
    last_played_card: Option<Card>,
}

impl Nim {
    pub fn new(player: Player, bet: i64) -> Self {
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

    pub fn add_machine(&mut self) -> Result<(), Error> {
        self.add_player(Player::new(None, true))?;
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
        self.current_player().hand.iter().all(|card| card.disabled)
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

    pub async fn bot_play(&mut self) -> Result<(), Error> {
        if !self.current_player().is_bot() {
            return Err("player is not a bot".into());
        }

        let available_cards: Vec<(usize, &Card)> = self
            .current_player()
            .hand
            .iter()
            .enumerate()
            .filter(|(_, card)| !card.disabled)
            .collect();

        if available_cards.is_empty() {
            return Err("bot has no available cards".into());
        }

        let min_card_index = || {
            available_cards
                .iter()
                .min_by_key(|(_, card)| card.value())
                .map(|(idx, _)| *idx)
                .unwrap()
        };

        let find_card_with_value_or_min = |value| {
            available_cards
                .iter()
                .find(|(_, card)| card.value() == value)
                .map(|(idx, _)| *idx)
                .unwrap_or_else(min_card_index)
        };

        let prioritize_one_index = || {
            if self.table_value() < 9 {
                let one_card = available_cards.iter().find(|(_, card)| card.value() == 1);

                if let Some((idx, _)) = one_card {
                    return *idx;
                }
            }

            let zero_card = available_cards.iter().find(|(_, card)| card.value() == 0);

            if let Some((idx, _)) = zero_card {
                return *idx;
            }

            min_card_index()
        };

        let index = if self.table_value() == 9 {
            min_card_index()
        } else {
            match self.table_value() % 4 {
                0 => find_card_with_value_or_min(1),
                1 => prioritize_one_index(),
                2 => find_card_with_value_or_min(3),
                3 => find_card_with_value_or_min(2),
                _ => min_card_index(),
            }
        };

        self.play_card(index).await?;
        tokio::time::sleep(Duration::from_secs(3)).await;
        Ok(())
    }

    pub async fn play_card(&mut self, index: usize) -> Result<(), Error> {
        {
            let card = self.mut_current_player().hand.get_mut(index).unwrap();
            card.disabled = true;
        }

        let card = self.current_player().hand.get(index).unwrap().clone();
        self.table_cards.push(card.clone());
        self.last_played_card = Some(card);
        Ok(())
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
