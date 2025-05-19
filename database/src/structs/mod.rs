pub mod club;
pub mod guild;
pub mod member;
pub mod system;
pub use system::System;

use crate::models::{AgentRelation, AuthorityId, ClubItemType, ItemType, JobModel, Quality, Role};
use sqlx::{FromRow, PgPool};
use std::collections::HashMap;
use types::cards::poker::Card;

#[derive(Clone, Debug)]
pub struct Item {
    pub id: Option<i32>,
    pub name: String,
    pub number: Option<i32>,
    pub usable: bool,
    pub item_type: ItemType,
    pub quality: Quality,
    pub two_handed: bool,
}

#[derive(Clone, Debug)]
pub struct ItemAmount {
    pub info: Item,
    pub amount: i32,
}

#[derive(Clone, Debug)]
pub struct Product {
    pub item: Item,
    pub price: i32,
    pub description: Option<String>,
    pub stock: Option<i32>,
}

#[derive(Clone, Debug)]
pub struct MemberState {
    pub can_claim_daily: bool,
    pub in_gamble: bool,
}

#[derive(Clone, Debug, FromRow)]
pub struct NormalRoles {
    pub roles: Vec<Role>,
}

#[derive(Clone, Debug, FromRow)]
pub struct RoleItem {
    pub item_type: ClubItemType,
    pub item_tr_key: String,
}

#[derive(Clone, Debug)]
pub struct ClubRole {
    pub id: Option<AuthorityId>,
    pub tr_key: String,
    pub range: Option<i32>,
    pub item: Option<RoleItem>,
}

#[derive(Clone, Debug)]
pub struct Club {
    pub id: i64,
    pub name: String,
    pub relation: Option<AgentRelation>,
    pub role: ClubRole,
}

#[derive(Clone, Debug, FromRow)]
pub struct NormalBalance {
    pub points: i32,
    pub bios: i64,
}

#[derive(Clone, Debug)]
pub struct Member {
    pub id: i64,
    pub balance: NormalBalance,
    pub inventory: HashMap<i32, ItemAmount>,
    pub job: Option<JobModel>,
    pub roles: Vec<Role>,
    pub state: MemberState,
    pub deck: Vec<Card>,
    pub club_id: Option<i64>,
}

impl From<ItemAmount> for Item {
    fn from(value: ItemAmount) -> Self {
        value.info
    }
}
