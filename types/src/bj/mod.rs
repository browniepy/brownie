use std::sync::atomic::AtomicU8;

pub mod player;
pub use player::{Dealer, Player, State};

use crate::cards::poker::Card;
use poise::serenity_prelude::User;

pub enum FinishReason {
    Timeout,
    FinalRound,
    PlayerLeave,
}

pub enum RoundResult {
    Draw,
    Win { state: State },
    Lose { bust: bool },
}

pub struct Blackjack {
    pub deck: Vec<Card>,
    pub player: Player,
    pub dealer: Dealer,
    pub timeout: Option<AtomicU8>,
    pub is_dealer_bust: bool,
    pub max_rounds: usize,
    pub round: usize,
}

impl Blackjack {
    pub fn new(user: User, bet: i32) -> Self {
        let player = Player::new(user, bet);

        Self {
            deck: Card::standart_deck(),
            player,
            dealer: Dealer::default(),
            timeout: None,
            is_dealer_bust: false,
            max_rounds: 7,
            round: 1,
        }
    }

    pub fn add_round(&mut self) {
        self.round += 1
    }

    pub fn finish(&self) -> bool {
        self.round == self.max_rounds
    }

    pub fn player_wins(&self) -> bool {
        match self.player.state {
            State::Bust => false, // Jugador pierde si se pasa
            _ => {
                if self.is_dealer_bust {
                    true // Jugador gana si el dealer se pasa
                } else {
                    let player_value = self.player.hand_value();
                    let dealer_value = self.dealer.hand_value(false);

                    player_value > dealer_value // Jugador gana si su puntaje es mayor
                }
            }
        }
    }

    pub fn round_result(&self) -> RoundResult {
        if self.player.is_bust() {
            return RoundResult::Lose { bust: true };
        }

        if self.player_wins() {
            return RoundResult::Win {
                state: self.player.state.clone(),
            };
        }

        if self.is_dealer_bust || self.dealer.hand_value(false) == self.player.hand_value() {
            return RoundResult::Draw;
        }

        return RoundResult::Lose { bust: false };
    }

    pub fn set_timeout(&mut self) {
        self.timeout = Some(AtomicU8::new(45));
    }

    pub fn decrement_timeout(&self) {
        use std::sync::atomic::Ordering;

        if let Some(timeout) = self.timeout.as_ref() {
            timeout.fetch_sub(1, Ordering::Relaxed);
        }
    }

    pub fn is_timeout(&self) -> bool {
        use std::sync::atomic::Ordering;

        self.timeout
            .as_ref()
            .map(|timeout| timeout.load(Ordering::Relaxed) == 0)
            .unwrap_or(false)
    }

    pub fn clear_hands(&mut self) {
        self.player.hand.clear();
        self.player.state = State::None;
        self.dealer.hand.clear();
    }

    pub fn deal_cards(&mut self) {
        self.player.hand.push(self.deck.pop().unwrap());
        self.dealer.hand.push(self.deck.pop().unwrap());

        self.player.hand.push(self.deck.pop().unwrap());
        self.dealer.hand.push(self.deck.pop().unwrap());
    }

    pub fn check_deck(&mut self) {
        if self.deck.len() < 10 {
            self.deck = Card::standart_deck();
        }
    }

    pub fn player_hit(&mut self) {
        let card = self.deck.pop().unwrap();
        self.player.hand.push(card);

        if self.player.hand_value() > 21 {
            self.player.state = State::Bust;
        }
    }

    pub fn dealer_hit(&mut self) {
        if self.dealer.hand_value(false) < 17 {
            self.dealer.hand.push(self.deck.pop().unwrap());

            if self.dealer.hand_value(false) > 21 {
                self.is_dealer_bust = true;
            }
        }
    }
}
