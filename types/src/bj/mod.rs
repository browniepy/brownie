mod player;
use std::{collections::HashMap, sync::atomic::AtomicU8};

pub use player::{Dealer, Player, State};

use crate::cards::poker::Card;
use poise::serenity_prelude::{User, UserId};

pub enum FinishReason {
    Timeout,
    FinalRound,
    EmptyPlayers,
}

pub struct RoundResult<'a> {
    pub winners: Option<Vec<&'a Player>>,
    pub losers: Option<Vec<&'a Player>>,
    pub draws: Option<Vec<&'a Player>>,
}

pub struct Blackjack {
    pub deck: Vec<Card>,
    pub players: HashMap<UserId, Player>,
    pub dealer: Dealer,
    pub timeout: Option<AtomicU8>,
    pub max_players: u8,
    pub is_dealer_bust: bool,
    pub max_rounds: usize,
    pub round: usize,
}

impl Blackjack {
    pub fn new(user: User, bet: i32, max_players: u8) -> Self {
        let player = Player::new(user, bet);

        let mut players = HashMap::new();
        players.insert(player.id, player);

        Self {
            deck: Card::standart_deck(),
            players,
            dealer: Dealer::default(),
            timeout: None,
            max_players,
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

    pub fn is_full(&mut self) -> bool {
        self.players.len() as u8 >= self.max_players
    }

    pub fn is_solo(&self) -> bool {
        self.players.len() == 1 && self.max_players == 1
    }

    pub fn all_stand(&mut self) -> bool {
        // check if all players are stand or Blackjack
        self.players
            .values_mut()
            .all(|player| player.is_stand() || player.is_blackjack_and_set())
    }

    pub fn all_bust(&mut self) -> bool {
        // check if all players are bust
        self.players.values_mut().all(|player| player.is_bust())
    }

    fn get_winners(&self) -> Vec<&Player> {
        let mut winners = Vec::new();

        for player in self.players.values() {
            // colapsed version
            if player.is_blackjack()
                || (!player.is_bust() && player.hand_value() > self.dealer.hand_value(false))
            {
                winners.push(player);
            }
        }

        winners
    }

    fn get_losers(&self) -> Vec<&Player> {
        let mut losers = Vec::new();

        for player in self.players.values() {
            if player.is_bust() || (player.hand_value() < self.dealer.hand_value(false)) {
                losers.push(player);
            }
        }

        losers
    }

    fn get_draws(&self) -> Vec<&Player> {
        let mut draws = Vec::new();

        for player in self.players.values() {
            if player.hand_value() == self.dealer.hand_value(false) && !player.is_bust() {
                draws.push(player);
            }
        }

        draws
    }

    pub fn get_results(&self) -> RoundResult {
        let winners = self.get_winners();
        let losers = self.get_losers();
        let draws = self.get_draws();

        RoundResult {
            winners: if winners.is_empty() {
                None
            } else {
                Some(winners)
            },
            losers: if losers.is_empty() {
                None
            } else {
                Some(losers)
            },
            draws: if draws.is_empty() { None } else { Some(draws) },
        }
    }

    pub fn add_player(&mut self, user: User, bet: i32) {
        let player = Player::new(user, bet);
        self.players.insert(player.id, player);
    }

    pub fn remove_player(&mut self, user: &User) {
        self.players.remove(&user.id);
    }

    pub fn get_player(&self, user: &User) -> Option<&Player> {
        self.players.get(&user.id)
    }

    pub fn get_player_mut(&mut self, user: &User) -> Option<&mut Player> {
        self.players.get_mut(&user.id)
    }

    pub fn get_multiplier(&self) -> i32 {
        match self.players.len() {
            1 => 0,
            2 => 10,
            _ => 20,
        }
    }

    fn reload_deck(&mut self) {
        self.deck = Card::standart_deck()
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
        for player in self.players.values_mut() {
            player.hand.clear();
            player.state = State::None;
        }
        self.dealer.hand.clear();
    }

    pub fn deal_cards(&mut self) {
        for player in self.players.values_mut() {
            player.hand.push(self.deck.pop().unwrap());
            player.hand.push(self.deck.pop().unwrap());

            player.is_blackjack_and_set();
        }

        self.dealer.hand.push(self.deck.pop().unwrap());
        self.dealer.hand.push(self.deck.pop().unwrap());
    }

    /// check if deck needs to be reloaded
    pub fn check_deck(&mut self) {
        if self.deck.len() < 10 {
            self.reload_deck();
        }
    }

    pub fn player_hit(&mut self, user: &User) {
        let card = self.deck.pop().unwrap();
        let player = self.get_player_mut(user).unwrap();

        player.hand.push(card);

        if player.hand_value() > 21 && !player.is_stand() && !player.is_blackjack_and_set() {
            player.state = State::Bust;
        }
    }

    pub fn is_player_in_game(&self, user: &User) -> bool {
        self.players.contains_key(&user.id)
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
