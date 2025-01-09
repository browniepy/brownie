pub mod blackjack;

pub mod cards;

pub mod oldmaid;

pub mod evaluate;
pub use evaluate::{EvaluatePoker, HandType};

pub mod airpoker;

pub mod rb;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
