pub use super::{super::Error, *};
use crate::models::{
    AuthorityId, ClubItemType, ItemInventory, ItemType, JobModel, Quality, RpgItemInventory,
};
use tokio::try_join;
use types::cards::poker::Card;

pub mod balance;
pub mod items;
pub mod points;
pub mod roles;
pub mod stats;
pub mod work;

impl Member {
    pub async fn build(pool: &PgPool, id: i64) -> Result<Self, Error> {
        sqlx::query!(
            "INSERT INTO member (id) VALUES ($1) ON CONFLICT (id) DO NOTHING",
            id
        )
        .execute(pool)
        .await?;

        let (balance, state, job, inventory, roles) = try_join!(
            NormalBalance::load(pool, id),
            MemberState::load(pool, id),
            JobModel::load(pool, id),
            normal_inventory(pool, id),
            roles(pool, id),
        )?;

        let club = sqlx::query!("SELECT club AS id FROM club_member WHERE member = $1;", id)
            .fetch_optional(pool)
            .await?;

        Ok(Self {
            id,
            balance,
            inventory,
            job,
            roles,
            state,
            deck: Card::standart_deck(),
            club_id: club.map(|club| club.id),
        })
    }

    pub fn reload_deck(&mut self) {
        self.deck = Card::standart_deck();
    }
}

impl JobModel {
    pub async fn load(pool: &PgPool, id: i64) -> Result<Option<Self>, Error> {
        let record = sqlx::query_as!(
            Self,
            "SELECT job.id, job.name,
            job.salary AS \"salary: Vec<i32>\",
            job.required_role AS \"required_role: Role\",
            job.required_points, job.cooldown
            FROM job INNER JOIN member ON job.id = member.job
            WHERE member.id = $1;",
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(record)
    }
}

impl AgentRelation {
    pub async fn load(pool: &PgPool, id: i64) -> Result<Option<Self>, Error> {
        let record = sqlx::query_as!(
            AgentRelation,
            "SELECT member, agent FROM agent_relation
            WHERE member = $1 OR agent = $1;",
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(record)
    }
}

pub async fn roles(pool: &PgPool, id: i64) -> Result<Vec<Role>, Error> {
    let record = sqlx::query!(
        "SELECT roles AS \"roles: Vec<Role>\"
            FROM member WHERE id = $1;",
        id
    )
    .fetch_one(pool)
    .await?;

    Ok(record.roles)
}

async fn normal_inventory(pool: &PgPool, id: i64) -> Result<HashMap<i32, ItemAmount>, Error> {
    let record = sqlx::query_as!(
        ItemInventory,
        "SELECT item.id, item.name, inventory.amount, item.usable, item.quality AS \"quality: Quality\", item.item_type AS \"item_type: ItemType\", victim
        FROM normal_inventory inventory
        INNER JOIN normal_item item ON inventory.item = item.id WHERE inventory.member = $1;",
        id
    )
    .fetch_all(pool)
    .await?;

    let inventory = record
        .iter()
        .map(|item| (item.id, item.clone().into()))
        .collect::<HashMap<i32, ItemAmount>>();

    Ok(inventory)
}

impl MemberState {
    pub async fn load(pool: &PgPool, id: i64) -> Result<Self, Error> {
        let can_claim_daily = sqlx::query!("SELECT can_claim_daily_reward($1);", id)
            .fetch_one(pool)
            .await?;

        Ok(Self {
            can_claim_daily: can_claim_daily.can_claim_daily_reward.unwrap(),
            in_gamble: false,
        })
    }
}

impl NormalBalance {
    pub async fn load(pool: &PgPool, id: i64) -> Result<Self, Error> {
        let normal = sqlx::query_as!(
            Self,
            "SELECT balance as bios, points
            FROM member WHERE id = $1;",
            id
        )
        .fetch_one(pool)
        .await?;

        Ok(normal)
    }
}
impl From<ItemInventory> for ItemAmount {
    fn from(value: ItemInventory) -> Self {
        Self {
            info: Item {
                id: Some(value.id),
                name: value.name,
                number: None,
                usable: value.usable,
                quality: value.quality,
                item_type: value.item_type,
                two_handed: false,
            },
            amount: value.amount,
        }
    }
}

impl From<RpgItemInventory> for ItemAmount {
    fn from(value: RpgItemInventory) -> Self {
        Self {
            info: Item {
                id: Some(value.id),
                name: value.name,
                number: None,
                usable: value.usable,
                quality: value.quality,
                item_type: value.item_type,
                two_handed: value.two_handed,
            },
            amount: value.amount,
        }
    }
}
