use crate::cards::poker::{Card, PokerValue};
use inflector::Inflector;
use poise::serenity_prelude::{User, UserId};
use rand::seq::SliceRandom;

#[derive(Debug)]
pub struct Player {
    pub id: UserId,
    pub name: String,
    pub hand: Vec<Card>,
    pub discarted_cards: Vec<Card>,
    pub confirmed_card_index: Option<usize>,
}

impl Player {
    pub fn new(user: &User) -> Self {
        let name = user.display_name().to_title_case();
        let hand = Vec::new();
        Self {
            id: user.id,
            name,
            hand,
            discarted_cards: Vec::new(),
            confirmed_card_index: None,
        }
    }

    pub fn shuffle_hand(&mut self) {
        self.hand.shuffle(&mut rand::thread_rng());
    }

    pub fn take_card(&mut self, index: usize) -> Card {
        self.hand.remove(index)
    }

    pub fn discard_pairs(&mut self) {
        let mut indices_to_remove = Vec::new();

        for i in 0..self.hand.len() {
            if indices_to_remove.contains(&i) {
                continue;
            }

            for j in (i + 1)..self.hand.len() {
                if indices_to_remove.contains(&j) {
                    continue;
                }

                if self.hand[i].value() == self.hand[j].value() {
                    indices_to_remove.push(i);
                    indices_to_remove.push(j);
                    break;
                }
            }
        }

        indices_to_remove.sort_by(|a, b| b.cmp(a));

        for index in indices_to_remove {
            self.discarted_cards.push(self.hand.remove(index));
        }
    }

    pub fn discard_card(&mut self, card: &Card) -> bool {
        if let Some(index) = self
            .hand
            .iter()
            .position(|c| c.key_name() == card.key_name())
        {
            self.discarted_cards.push(self.hand.remove(index));
            self.discarted_cards.push(card.clone());
            true
        } else {
            false
        }
    }
}
