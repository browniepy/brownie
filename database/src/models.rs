use sqlx::{FromRow, Type};

#[derive(Debug, Clone, Type, PartialEq)]
#[sqlx(type_name = "roles_enum")]
pub enum Role {
    Member,
    Referee,
    Leader,
    Baku,
    Slave,
    User,
    Pochi,
}

#[derive(Debug, Clone, FromRow)]
pub struct Debt {
    pub to: Option<i64>,
    pub amount: Option<i32>,
}

#[derive(Debug, Clone, FromRow)]
pub struct LbMember {
    pub id: i64,
    pub balance: i32,
    pub points: i32,
    pub level: i32,
}

#[derive(Debug, Clone, FromRow)]
pub struct JobModel {
    pub name: String,
    pub description: Option<String>,
    pub salary_range: Option<Vec<i32>>,
    pub required_role: Option<Role>,
    pub required_level: i32,
    pub cooldown: i32,
}

impl Default for JobModel {
    fn default() -> Self {
        Self {
            name: String::default(),
            description: None,
            salary_range: None,
            required_role: None,
            required_level: 1,
            cooldown: 3600,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct MemberModel {
    pub balance: i32,
    pub roles: Option<Vec<Role>>,
    pub points: i32,
    pub referee_range: Option<i32>,
    pub personal_referee: Option<i64>,
    pub profile_text: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct StatModel {
    pub game: String,
    pub victories: i32,
    pub defeats: i32,
    pub victory_text: Option<String>,
    pub defeat_text: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct ItemInventory {
    pub id: Option<i32>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub amount: Option<i32>,
}

#[derive(Debug, Clone, FromRow)]
pub struct ItemShop {
    pub id: Option<i32>,
    pub name: Option<String>,
    pub price: Option<i32>,
    pub description: Option<String>,
}
