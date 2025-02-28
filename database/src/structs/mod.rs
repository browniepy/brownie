mod member;
pub mod system;

#[derive(Clone, Debug, PartialEq)]
pub enum Gamble {
    AirPoker { id: u64 },
    Contradict { id: u64 },
    None,
}

pub use member::Member;
pub use system::System;
