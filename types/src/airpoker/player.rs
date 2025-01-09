use crate::{cards::air::SteelCard, Error};
use inflector::Inflector;
use poise::serenity_prelude::{Message, User, UserId};
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug)]
pub struct AirBios {
    // duration of the air tank
    pub duration: AtomicU8,
    // field to check if the bios is active
    pub is_active: bool,
}

#[derive(Debug)]
pub struct Player {
    pub id: UserId,
    pub name: String,
    // hand of steel cards
    pub hand: Vec<SteelCard>,
    // amount of air tanks
    pub air_bios: Vec<AirBios>,
    // field for selected card
    pub selected_card: Option<SteelCard>,
    pub is_alive: bool,
    // field for the player bet
    pub bet: u8,
    pub ephemeral_message: Option<Message>,
}

// default 25 air bios with 30 seconds of air
impl Player {
    pub fn new(user: User) -> Self {
        let name = user.display_name().to_title_case();

        let mut air_bios = Vec::new();

        for _ in 0..25 {
            air_bios.push(AirBios {
                duration: AtomicU8::new(30),
                is_active: false,
            });
        }

        Self {
            id: user.id,
            name,
            hand: Vec::new(),
            air_bios,
            selected_card: None,
            is_alive: true,
            bet: 0,
            ephemeral_message: None,
        }
    }

    pub fn set_ephemeral(&mut self, message: Message) {
        self.ephemeral_message = Some(message);
    }

    pub fn get_ephemeral(&self) -> Result<Message, Error> {
        if self.ephemeral_message.is_none() {
            Err("empty ephemeral message".into())
        } else {
            Ok(self.ephemeral_message.clone().unwrap())
        }
    }

    pub fn select_random_card(&mut self) {
        use rand::thread_rng;
        use rand::Rng;

        let mut rng = thread_rng();
        let index = rng.gen_range(0..self.hand.len());
        self.selected_card = Some(self.hand.remove(index));
    }

    pub fn select_card(&mut self, index: usize) {
        self.selected_card = Some(self.hand.remove(index));
    }

    pub fn reset_selected_card(&mut self) {
        self.selected_card = None;
    }

    // rework of consume_air_bio
    pub fn consume_air_bio(&mut self) -> bool {
        if let Some(active_tank) = self.air_bios.iter_mut().find(|b| b.is_active) {
            let duration = active_tank.duration.load(Ordering::Relaxed);

            if duration > 0 {
                active_tank.duration.fetch_sub(1, Ordering::Relaxed);
                true
            } else {
                self.clean_air_bios();

                if let Some(next_tank) = self.air_bios.iter_mut().find(|b| !b.is_active) {
                    next_tank.is_active = true;
                    true
                } else {
                    self.is_alive = false;
                    false
                }
            }
        } else {
            if let Some(next_tank) = self.air_bios.iter_mut().find(|b| !b.is_active) {
                next_tank.is_active = true;
                true
            } else {
                self.is_alive = false;
                false
            }
        }
    }

    pub fn consume_air_bi(&mut self) -> bool {
        if let Some(active_tank) = self.air_bios.iter_mut().find(|tank| tank.is_active) {
            let duration_val = active_tank.duration.load(Ordering::Relaxed);
            if duration_val > 0 {
                active_tank.duration.fetch_sub(1, Ordering::Relaxed);
                true
            } else {
                // if the active tank ran out of oxygen, check for another one.
                if self.air_bios.iter().filter(|tank| !tank.is_active).count() > 0 {
                    let mut next_tank_found = false;
                    for tank in self.air_bios.iter_mut() {
                        if !tank.is_active {
                            tank.is_active = true;
                            next_tank_found = true;
                            break;
                        }
                    }
                    if next_tank_found {
                        true
                    } else {
                        self.is_alive = false;
                        false
                    }
                } else {
                    self.is_alive = false;
                    false
                }
            }
        } else {
            // find to activate a tank
            if let Some(tank) = self.air_bios.iter_mut().find(|tank| !tank.is_active) {
                tank.is_active = true;
                true
            } else {
                self.is_alive = false;
                false
            }
        }
    }

    pub fn get_active_tank_duration(&self) -> u8 {
        if let Some(active_tank) = self.air_bios.iter().find(|tank| tank.is_active) {
            return active_tank.duration.load(Ordering::Relaxed);
        }
        0
    }

    pub fn get_betable_air_bios(&self) -> usize {
        let mut available_tanks = self.air_bios.len();
        if self.air_bios.iter().any(|tank| tank.is_active) {
            available_tanks -= 1;
        }
        available_tanks
    }

    pub fn remove_air_bios(&mut self, amount: usize) {
        let mut amount_to_remove = amount;
        self.air_bios.retain(|tank| {
            if !tank.is_active && amount_to_remove > 0 {
                amount_to_remove -= 1;
                false
            } else {
                true
            }
        });
    }

    // delete used air bios without duration
    pub fn clean_air_bios(&mut self) {
        // this will conserve the air bios with duration
        self.air_bios
            .retain(|tank| tank.duration.load(Ordering::Relaxed) > 0);
    }
}
