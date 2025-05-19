use chrono::{DateTime, Utc};
use sqlx::{FromRow, Type};

#[derive(Debug, Clone, Type, PartialEq)]
#[sqlx(type_name = "role")]
pub enum Role {
    Member,
    Referee,
    Leader,
    Player,
}

#[derive(Debug, Clone, Type, PartialEq, poise::ChoiceParameter)]
#[sqlx(type_name = "club_item_type")]
pub enum ClubItemType {
    Membership,
    Agent,
}

#[derive(Debug, Clone, Type, PartialEq)]
#[sqlx(type_name = "authority_id")]
pub enum AuthorityId {
    Leader,
    Agent,
    Member,
}

#[derive(Debug, Clone, Type, PartialEq)]
#[sqlx(type_name = "rpg_role")]
pub enum RpgRole {
    King,
    Knight,
}

#[derive(Debug, Clone, Type, PartialEq)]
#[sqlx(type_name = "item_type")]
pub enum ItemType {
    Equipment,
    Tool,
    Material,
    Quest,
    Misc,
    Consumable,
}

#[derive(Debug, Clone, Type, PartialEq)]
#[sqlx(type_name = "armor_type")]
pub enum ArmorType {
    Head,
    Chest,
    Legs,
    Boots,
    Neck,
    Ring,
}

#[derive(Debug, Clone, Type, PartialEq, poise::ChoiceParameter)]
#[sqlx(type_name = "club_type")]
pub enum ClubType {
    Club,
    Academy,
    Organization,
    Mafia,
    Fundation,
    Group,
}

#[derive(Debug, Clone, Type, PartialEq)]
#[sqlx(type_name = "perm")]
pub enum ClubRolePerm {
    ManageRoles,
    ManageMembers,
    ManageBank,
    ManageClub,
    InviteMembers,
    All,
}

#[derive(Debug, Clone, Type, PartialEq)]
#[sqlx(type_name = "tool_type")]
pub enum Tool {
    Weapon,
    Shield,
    Accessory,
    Pickaxe,
    Axe,
}

#[derive(Debug, Clone, Type, PartialEq)]
#[sqlx(type_name = "quality")]
pub enum Quality {
    Common,
    Normal,
    Epic,
    Masterpiece,
}

#[derive(Debug, Clone, Type, PartialEq)]
#[sqlx(type_name = "rpg_class")]
pub enum RpgClass {
    Mage,
    Warrior,
    Archer,
}

#[derive(Debug, Clone, FromRow)]
pub struct BoardMember {
    pub id: i64,
    pub balance: i64,
    pub points: i32,
}

#[derive(Debug, Clone, FromRow)]
pub struct JobModel {
    pub id: i32,
    pub name: String,
    pub salary: Vec<i32>,
    pub required_role: Option<Role>,
    pub required_points: i32,
    pub cooldown: i32,
}

impl Default for JobModel {
    fn default() -> Self {
        Self {
            id: 0,
            name: "none-work".to_string(),
            salary: vec![700, 900],
            required_role: None,
            required_points: 0,
            cooldown: 300,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct RefereeRelation {
    pub member: i64,
    pub referee: Option<i64>,
}

#[derive(Debug, Clone, FromRow)]
pub struct AgentRelation {
    pub member: i64,
    pub agent: Option<i64>,
}

impl RefereeRelation {
    pub fn relation(member: i64, referee: i64) -> Self {
        Self {
            member,
            referee: Some(referee),
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct MemberModel {
    pub balance: i64,
    pub roles: Vec<Role>,
    pub referee_range: Option<i32>,
    pub registered_at: DateTime<Utc>,
    pub points: i32,
}

#[derive(Debug, Clone, FromRow)]
pub struct Item {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct ItemInventory {
    pub id: i32,
    pub name: String,
    pub amount: i32,
    pub usable: bool,
    pub item_type: ItemType,
    pub quality: Quality,
    pub victim: Option<i64>,
}

#[derive(Debug, Clone, FromRow)]
pub struct RpgItemInventory {
    pub id: i32,
    pub name: String,
    pub amount: i32,
    pub usable: bool,
    pub tool: Option<Tool>,
    pub item_type: ItemType,
    pub armor_type: Option<ArmorType>,
    pub two_handed: bool,
    pub quality: Quality,
}

#[derive(Debug, Clone, FromRow)]
pub struct ItemShop {
    pub id: i32,
    pub name: String,
    pub price: i32,
}
