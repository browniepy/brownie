pub mod member;
pub mod system;
pub use system::System;

use crate::models::{
    AgentRelation, ArmorType, AuthorityId, ClubItemType, ItemType, JobModel, Quality, Role,
    RpgRole, Tool,
};
use sqlx::PgPool;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Item {
    pub id: Option<i32>,
    pub name: String,
    pub number: Option<i32>,
    pub usable: bool,
    pub item_type: ItemType,
    pub quality: Quality,
    pub armor_type: Option<ArmorType>,
    pub tool_type: Option<Tool>,
    pub two_handed: bool,
}

#[derive(Clone, Debug)]
pub struct InventoryItem {
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
    pub in_rpg: bool,
}

#[derive(Clone, Debug)]
pub struct Inventories {
    pub normal: NormalInventory,
    pub rpg: Option<RpgInventory>,
}

#[derive(Clone, Debug)]
pub struct NormalInventory {
    pub items: HashMap<i32, InventoryItem>,
}

#[derive(Clone, Debug)]
pub struct RpgInventory {
    pub items: HashMap<i32, InventoryItem>,
}

#[derive(Clone, Debug)]
pub struct Roles {
    pub normal: NormalRoles,
    pub rpg: Option<RpgRoles>,
}

#[derive(Clone, Debug)]
pub struct NormalRoles {
    pub roles: Vec<Role>,
}

#[derive(Clone, Debug)]
pub struct RpgRoles {
    pub roles: Vec<RpgRole>,
}

#[derive(Clone, Debug)]
pub struct RoleItem {
    pub item_type: ClubItemType,
    pub tr_key: String,
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

#[derive(Clone, Debug)]
pub struct NormalBalance {
    pub points: i32,
    pub yn: i64,
}

#[derive(Clone, Debug)]
pub struct RpgBalance {
    pub bios: i64,
    pub exp: i32,
    pub level: i32,
}

#[derive(Clone, Debug)]
pub struct Balance {
    pub normal: NormalBalance,
    pub rpg: Option<RpgBalance>,
}

#[derive(Clone, Debug)]
pub struct Member {
    pub id: i64,
    pub balance: Balance,
    pub inventories: Inventories,
    pub job: Option<JobModel>,
    pub roles: Roles,
    pub state: MemberState,
    pub club: Option<Club>,
}

impl From<InventoryItem> for Item {
    fn from(value: InventoryItem) -> Self {
        value.info
    }
}
