use inflector::Inflector;
use itertools::Itertools;
use poise::serenity_prelude::{Message, User, UserId};

pub enum Role {
    Defender,
    Attacker,
    None,
}

impl Role {
    pub fn list() -> Vec<Self> {
        vec![Self::Defender, Self::Attacker]
    }
}

#[derive(Hash, Clone, PartialEq, Eq)]
pub enum State {
    Cut,
    Tased,
    Shot,
}

impl State {
    pub fn danger(&self) -> usize {
        match *self {
            Self::Cut => 20,
            Self::Tased => 20,
            Self::Shot => 40,
        }
    }
}

pub struct Player {
    pub id: UserId,
    pub name: String,
    pub bios: usize,
    pub current_bet: usize,
    pub role: Role,
    pub states: Vec<State>,
    pub anxiety: usize,
    pub ephemeral: Option<Message>,
}

impl Player {
    pub fn new(user: &User) -> Self {
        Self {
            id: user.id,
            name: user.display_name().to_title_case(),
            bios: 10000,
            current_bet: 0,
            role: Role::None,
            states: Vec::new(),
            anxiety: 0,
            ephemeral: None,
        }
    }

    pub fn set_ephemeral(&mut self, message: Message) {
        self.ephemeral = Some(message);
    }

    pub fn delete_ephemeral(&mut self) {
        self.ephemeral = None;
    }

    pub fn is_tased(&self) -> bool {
        self.states.contains(&State::Tased)
    }

    pub fn is_shot(&self) -> bool {
        self.states.contains(&State::Shot)
    }

    pub fn is_cut(&self) -> bool {
        self.states.contains(&State::Cut)
    }

    pub fn set_role(&mut self, role: Role) {
        self.role = role;
    }

    pub fn invert_role(&mut self) {
        self.role = match self.role {
            Role::Defender => Role::Attacker,
            Role::Attacker => Role::Defender,
            Role::None => Role::None,
        }
    }

    pub fn add_state(&mut self, state: State) {
        self.states.push(state);

        self.states = self
            .states
            .clone()
            .into_iter()
            .unique()
            .collect::<Vec<State>>();
    }

    pub fn sanity(&self) -> usize {
        self.states.iter().map(|state| state.danger()).sum()
    }

    pub fn bet(&mut self, amount: usize) {
        self.current_bet = amount;
    }

    pub fn confirm_bet(&mut self) {
        self.bios -= self.current_bet;
    }

    pub fn reset_bet(&mut self) {
        self.current_bet = 0;
    }

    pub fn add_anxiety(&mut self, amount: usize) {
        self.anxiety += amount;
    }
}
