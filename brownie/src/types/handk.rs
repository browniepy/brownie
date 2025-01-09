use super::{DropCheck, DropRole, DropState, NearDeath};
use chrono::{DateTime, Duration, Timelike, Utc};
use inflector::Inflector;
use poise::serenity_prelude::{ComponentInteraction, User, UserId};
use rand::{seq::SliceRandom, thread_rng, Rng};
use std::{collections::HashMap, time::Instant};
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct Player {
    pub id: UserId,
    pub name: String,
    pub role: DropRole,
    pub wasted_time: u64,
    pub reanimations: u64,
    pub death_time: u64,
}

#[derive(Debug, Clone)]
pub struct Dth {
    pub players: HashMap<UserId, Player>,
    pub drop_chrono: Option<Instant>,
    pub state: DropState,
    pub rounds: usize,
    pub deadline: Option<DateTime<Utc>>,
    pub round_timeout: Option<std::time::Duration>,
    pub round_income: Option<std::time::Duration>,
    pub last_inter: Option<ComponentInteraction>,
}

impl Player {
    pub fn build(role: DropRole, name: String, id: UserId) -> Self {
        Self {
            id,
            role,
            wasted_time: 0,
            name,
            reanimations: 0,
            death_time: 0,
        }
    }

    pub async fn inject_drug(&mut self) -> NearDeath {
        sleep(std::time::Duration::from_secs(10)).await;

        let mut rng = rand::thread_rng();
        let live_chance: u64 = rng.gen_range(0..=100);

        if live_chance > self.calculate_death_chance() {
            self.reanimations += 1;

            self.death_time += self.wasted_time;
            self.wasted_time = 0;
            NearDeath::Alive
        } else {
            NearDeath::Death
        }
    }

    fn calculate_death_chance(&self) -> u64 {
        let factor = match self.reanimations {
            0 => 0,
            1 => 35,
            2 => 70,
            _ => 100,
        };

        factor.min(100)
    }
}

impl Dth {
    pub fn build(a: User, b: User) -> Self {
        let mut roles = vec![DropRole::Dropper, DropRole::Checker];
        roles.shuffle(&mut thread_rng());

        let mut players = HashMap::new();

        players.insert(
            a.id,
            Player::build(roles.pop().unwrap(), a.name.to_title_case(), a.id),
        );
        players.insert(
            b.id,
            Player::build(roles.pop().unwrap(), b.name.to_title_case(), b.id),
        );

        Self {
            players,
            drop_chrono: None,
            state: DropState::Hand,
            rounds: 0,
            deadline: None,
            round_timeout: None,
            round_income: None,
            last_inter: None,
        }
    }

    pub fn set_inter(&mut self, inter: ComponentInteraction) {
        self.last_inter = Some(inter);
    }

    pub async fn next_round_timer(&self) {
        let now = Utc::now();
        let next_round_start = self.next_round_date();
        let dur_until_next_round = next_round_start.signed_duration_since(now);

        if dur_until_next_round > Duration::zero() {
            let sleep_duration = dur_until_next_round.to_std().unwrap();
            sleep(sleep_duration).await;
        }
    }

    pub fn round_start_state(&mut self) {
        self.deadline = Some(Utc::now() + Duration::minutes(1));
        self.round_timeout = Some(Duration::minutes(1).to_std().unwrap());
        self.state = DropState::Hand;
    }

    pub fn round_end_state(&mut self) {
        self.deadline = None;
        self.round_timeout = None;
        self.state = DropState::Hand;
    }

    pub fn next_round_date(&self) -> DateTime<Utc> {
        let now = Utc::now();
        let mut next_minute = now.minute() + 1;
        let mut next_hour = now.hour();

        if now.second() > 58 {
            next_minute += 1;
        }

        if next_minute >= 60 {
            next_minute = 0;
            next_hour += 1;
            if next_hour >= 24 {
                next_hour = 0;
            }
        }

        now.with_hour(next_hour)
            .unwrap()
            .with_minute(next_minute)
            .unwrap()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap()
    }

    // Check if the round time expired
    pub fn round_expired(&self) -> bool {
        match self.deadline {
            Some(deadline) => Utc::now() > deadline,
            None => false,
        }
    }

    // Init the chrono, changes the state to Dropped
    pub fn droph(&mut self) {
        self.drop_chrono = Some(Instant::now());
        self.state = DropState::Dropped;
    }

    pub fn check(&mut self) -> DropCheck {
        match self.state {
            DropState::Hand => {
                self.get_mut_player(DropRole::Checker).wasted_time += 60;
                DropCheck::Failed
            }
            DropState::Dropped => {
                let elapsed = self.drop_chrono.unwrap().elapsed().as_secs();

                self.get_mut_player(DropRole::Checker).wasted_time += elapsed;
                DropCheck::Sucess(elapsed)
            }
        }
    }

    // Obtain player with specific role
    pub fn get_player(&self, role: DropRole) -> &Player {
        self.players
            .values()
            .find(|player| player.role == role)
            .unwrap()
    }

    pub fn get_mut_player(&mut self, role: DropRole) -> &mut Player {
        self.players
            .values_mut()
            .find(|player| player.role == role)
            .unwrap()
    }

    pub fn swap_roles(&mut self) {
        let mut roles = self
            .players
            .values()
            .map(|player| player.role)
            .collect::<Vec<DropRole>>();
        roles.reverse();

        for (i, (_id, player)) in self.players.iter_mut().enumerate() {
            player.role = roles[i];
        }
    }
}
