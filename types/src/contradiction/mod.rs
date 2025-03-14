mod player;
pub use player::{Player, Role, State};

use poise::serenity_prelude::UserId;
use rand::{seq::SliceRandom, thread_rng};

pub enum ShieldEnum {
    Iron,
    Wood,
    Rubber,
}

pub enum WeaponEnum {
    Katana,
    Gun,
    Taser,
}

pub struct Weapon {
    pub weapon: WeaponEnum,
    pub used: bool,
}

pub struct Shield {
    pub shield: ShieldEnum,
    pub used: bool,
}

pub enum Level {
    Low,
    Medium,
    High,
}

pub enum Reaction {
    Deviated,
    Shot {
        shield: ShieldEnum,
    },

    Pierced {
        shield: ShieldEnum,
        level: Level,
    },

    Tased {
        shield: ShieldEnum,
        level: Level,
    },

    Stopped {
        weapon: WeaponEnum,
        shield: ShieldEnum,
    },
}

pub struct RoundInfo {
    pub game: usize,
    pub round: usize,
}

impl Weapon {
    pub fn new(w: WeaponEnum) -> Self {
        Self {
            weapon: w,
            used: false,
        }
    }

    fn list() -> Vec<Self> {
        vec![
            Self::new(WeaponEnum::Katana),
            Self::new(WeaponEnum::Gun),
            Self::new(WeaponEnum::Taser),
        ]
    }

    pub fn name(&self) -> &str {
        match self.weapon {
            WeaponEnum::Katana => "katana",
            WeaponEnum::Gun => "gun",
            WeaponEnum::Taser => "taser",
        }
    }
}

impl Shield {
    pub fn new(s: ShieldEnum) -> Self {
        Self {
            shield: s,
            used: false,
        }
    }

    fn list() -> Vec<Self> {
        vec![
            Self::new(ShieldEnum::Iron),
            Self::new(ShieldEnum::Wood),
            Self::new(ShieldEnum::Rubber),
        ]
    }

    pub fn name(&self) -> &str {
        match self.shield {
            ShieldEnum::Iron => "iron",
            ShieldEnum::Wood => "wood",
            ShieldEnum::Rubber => "rubber",
        }
    }
}

impl RoundInfo {
    fn new() -> Self {
        Self { game: 1, round: 1 }
    }

    pub fn setup_next(&mut self) {
        self.round = 1;
        self.game += 1;
    }

    pub fn add_round(&mut self) {
        self.round += 1;
    }
}

pub struct Contradiction {
    pub players: Vec<Player>,
    pub weapons: Vec<Weapon>,
    pub shields: Vec<Shield>,
    pub already_bet: Vec<UserId>,
    pub already_selected: Vec<UserId>,
    pub selected_weapon: Option<usize>,
    pub selected_shield: Option<usize>,
    pub round_info: RoundInfo,
}

pub trait Battle {
    fn battle(&mut self) -> Reaction;
}

impl Battle for Contradiction {
    fn battle(&mut self) -> Reaction {
        let weapon = &self
            .weapons
            .get(self.selected_weapon.unwrap())
            .unwrap()
            .weapon;
        let shield = &self
            .shields
            .get(self.selected_shield.unwrap())
            .unwrap()
            .shield;

        let (state, reaction) = match (weapon, shield) {
            // Gun weapon
            (WeaponEnum::Gun, ShieldEnum::Iron) => {
                let less_better = self.less_bet_player();

                if less_better.is_tased() || less_better.is_shot() {
                    less_better.add_anxiety(40);
                    (
                        Some(State::Shot),
                        Reaction::Shot {
                            shield: ShieldEnum::Iron,
                        },
                    )
                } else {
                    (None, Reaction::Deviated)
                }
            }
            (WeaponEnum::Gun, ShieldEnum::Wood) => {
                let less_better = self.less_bet_player();

                less_better.add_anxiety(50);
                (
                    Some(State::Shot),
                    Reaction::Shot {
                        shield: ShieldEnum::Wood,
                    },
                )
            }
            (WeaponEnum::Gun, ShieldEnum::Rubber) => {
                let less_better = self.less_bet_player();

                less_better.add_anxiety(45);
                (
                    Some(State::Shot),
                    Reaction::Shot {
                        shield: ShieldEnum::Rubber,
                    },
                )
            }

            // Katana weapon
            (WeaponEnum::Katana, ShieldEnum::Iron) => (
                None,
                Reaction::Stopped {
                    weapon: WeaponEnum::Katana,
                    shield: ShieldEnum::Iron,
                },
            ),
            (WeaponEnum::Katana, ShieldEnum::Wood) => {
                let less_better = self.less_bet_player();

                less_better.add_anxiety(30);
                (
                    Some(State::Cut),
                    Reaction::Pierced {
                        shield: ShieldEnum::Wood,
                        level: Level::Medium,
                    },
                )
            }
            (WeaponEnum::Katana, ShieldEnum::Rubber) => {
                let less_better = self.less_bet_player();

                less_better.add_anxiety(20);
                (
                    Some(State::Cut),
                    Reaction::Pierced {
                        shield: ShieldEnum::Rubber,
                        level: Level::Low,
                    },
                )
            }

            // Taser weapon
            (WeaponEnum::Taser, ShieldEnum::Iron) => {
                let less_better = self.less_bet_player();

                less_better.add_anxiety(30);
                (
                    Some(State::Tased),
                    Reaction::Tased {
                        shield: ShieldEnum::Iron,
                        level: Level::High,
                    },
                )
            }
            (WeaponEnum::Taser, ShieldEnum::Wood) => {
                let less_better = self.less_bet_player();

                less_better.add_anxiety(10);
                (
                    None,
                    Reaction::Tased {
                        shield: ShieldEnum::Wood,
                        level: Level::Low,
                    },
                )
            }
            (WeaponEnum::Taser, ShieldEnum::Rubber) => (
                None,
                Reaction::Stopped {
                    weapon: WeaponEnum::Taser,
                    shield: ShieldEnum::Rubber,
                },
            ),
        };

        if let Some(state) = state {
            let less_better = self.less_bet_player();
            less_better.add_state(state);
        }

        reaction
    }
}

impl Contradiction {
    pub fn new(players: Vec<Player>) -> Self {
        Self {
            players,
            weapons: Weapon::list(),
            shields: Shield::list(),
            already_bet: Vec::with_capacity(2),
            already_selected: Vec::with_capacity(2),
            selected_weapon: None,
            selected_shield: None,
            round_info: RoundInfo::new(),
        }
    }

    pub fn setup_next_round(&mut self) {
        self.reset_objects();
        self.invert_roles();
    }

    /// check if the game has to end
    pub fn to_end(&self) -> bool {
        self.get_loser().is_some()
    }

    /// check if only one shield and weapon are left
    pub fn only_one_object_left(&self) -> bool {
        self.weapons.len() == 1 && self.shields.len() == 1
    }

    pub fn init_roles(&mut self) {
        let mut rng = thread_rng();

        let mut roles = Role::list();
        roles.shuffle(&mut rng);

        for player in self.players.iter_mut() {
            player.set_role(roles.pop().unwrap());
        }
    }

    fn invert_roles(&mut self) {
        for player in self.players.iter_mut() {
            player.invert_role();
        }
    }

    pub fn reset_bets(&mut self) {
        for player in self.players.iter_mut() {
            player.reset_bet();
            self.already_bet.clear();
        }
    }

    pub fn all_bet(&self) -> bool {
        self.players.len() == self.already_bet.len()
    }

    pub fn all_selected(&self) -> bool {
        self.selected_weapon.is_some() && self.selected_shield.is_some()
    }

    pub fn is_bet_draw(&self) -> bool {
        let p1 = self.players.first().unwrap();
        let p2 = self.players.last().unwrap();

        p1.current_bet == p2.current_bet
    }

    pub fn check_zero_bios(&mut self) {
        for player in self.players.iter_mut() {
            if player.bios == 0 {
                player.current_bet = 0;
            }
        }
    }

    pub fn select_weapon(&mut self, index: usize) {
        self.selected_weapon = Some(index);
    }

    pub fn select_shield(&mut self, index: usize) {
        self.selected_shield = Some(index);
    }

    pub fn delete_stock(&mut self) {
        let weapon = self.weapons.get_mut(self.selected_weapon.unwrap()).unwrap();
        weapon.used = true;

        let shield = self.shields.get_mut(self.selected_shield.unwrap()).unwrap();
        shield.used = true;
    }

    pub fn reset_selections(&mut self) {
        self.selected_weapon = None;
        self.selected_shield = None;
    }

    fn reset_objects(&mut self) {
        self.weapons = Weapon::list();
        self.shields = Shield::list();
    }

    pub fn get_player(&self, id: UserId) -> Option<&Player> {
        self.players.iter().find(|player| player.id == id)
    }

    pub fn get_mut_player(&mut self, id: UserId) -> Option<&mut Player> {
        self.players.iter_mut().find(|player| player.id == id)
    }

    pub fn get_winner(&self) -> Option<&Player> {
        self.players.iter().find(|player| player.anxiety < 100)
    }

    pub fn get_loser(&self) -> Option<&Player> {
        self.players.iter().find(|player| player.anxiety >= 100)
    }

    pub fn less_bet_player(&mut self) -> &mut Player {
        if self.is_bet_draw() {
            self.players
                .iter_mut()
                .find(|player| player.role == Role::Defender)
                .unwrap()
        } else {
            self.players
                .iter_mut()
                .filter(|player| self.already_bet.contains(&player.id))
                .min_by_key(|player| player.current_bet)
                .unwrap()
        }
    }

    pub fn greater_bet_player(&mut self) -> &mut Player {
        if self.is_bet_draw() {
            self.players
                .iter_mut()
                .find(|player| player.role == Role::Attacker)
                .unwrap()
        } else {
            self.players
                .iter_mut()
                .filter(|player| self.already_bet.contains(&player.id))
                .max_by_key(|player| player.current_bet)
                .unwrap()
        }
    }
}
