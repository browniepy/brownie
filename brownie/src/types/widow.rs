use inflector::Inflector;
use poise::serenity_prelude::User;
use types::cards::poker::Card;

pub enum State {
    WidowLap,
    NormalLap,
    FinalTurn,
}

pub struct Player {
    pub name: String,
    pub cards: Vec<Card>,
}

pub struct Widow {
    pub players: Vec<Player>,
    pub widow_hand: Vec<Card>,
    pub state: State,
    pub turn_num: usize,
    pub round_timeout: Option<std::time::Duration>,
}

impl Player {
    pub fn new(user: User) -> Self {
        let name = user.display_name().to_table_case();

        Self {
            name,
            cards: Vec::new(),
        }
    }
}

impl Widow {
    pub fn new(p1: Player, p2: Player) -> Self {
        Self {
            players: vec![p1, p2],
            widow_hand: Vec::new(),
            state: State::WidowLap,
            turn_num: 1,
            round_timeout: None,
        }
    }

    pub fn trigger_timeout(&mut self) {
        self.round_timeout = Some(chrono::Duration::seconds(15).to_std().unwrap());
    }

    pub fn deal_cards(&mut self) {
        let mut deck = Card::standart_deck();

        for player in self.players.iter_mut() {
            player.cards = deck.drain(0..5).collect();
        }

        self.widow_hand = deck.drain(0..5).collect();
    }

    pub fn mut_current_player(&mut self) -> &mut Player {
        self.players.first_mut().unwrap()
    }

    pub fn current_player(&self) -> &Player {
        self.players.first().unwrap()
    }

    pub fn get_rival(&self) -> &Player {
        self.players.last().unwrap()
    }

    pub fn next_player(&mut self) {
        let player = self.players.remove(0);
        self.players.push(player);
    }

    pub fn change_card(&mut self, index_player: usize, index_widow: usize) {
        let current_player = self.players.first_mut().unwrap();

        std::mem::swap(
            &mut current_player.cards[index_player],
            &mut self.widow_hand[index_widow],
        );
    }
}
