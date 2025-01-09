use std::collections::HashMap;

use crate::Card;

use super::Member;

#[derive(Eq, Hash, PartialEq)]
pub struct TableId(pub i32);

#[derive(Eq, Hash, PartialEq)]
pub struct PlayerId(pub i64);

pub struct Blackjack {
    pub deck: Vec<Card>,
    pub players: HashMap<PlayerId, Member>,
}

pub struct Base {
    pub blackjack: HashMap<TableId, Blackjack>,
}
