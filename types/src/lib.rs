pub mod blackjack;

pub mod cards;

pub mod oldmaid;

pub mod evaluate;

use std::collections::HashMap;
use tokio::sync::Mutex;

pub use evaluate::{EvaluatePoker, HandType};

pub mod airpoker;
pub use airpoker::AirPoker;

pub mod rb;

pub mod nim_type_zero;
pub use nim_type_zero::Nim;

#[derive(Default)]
pub struct Rooms {
    pub nim: Mutex<HashMap<u64, Nim>>,
    pub airpoker: Mutex<HashMap<u64, AirPoker>>,
}

pub mod contradiction;

pub mod dices;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
