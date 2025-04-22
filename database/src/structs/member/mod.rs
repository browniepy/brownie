pub use super::{super::Error, *};
use crate::models::{
    ArmorType, AuthorityId, ClubItemType, ItemInventory, ItemType, JobModel, Quality,
    RpgItemInventory, RpgRole, Tool,
};
use tokio::try_join;

pub mod balance;
pub mod items;
pub mod points;
pub mod roles;
pub mod stats;
pub mod work;

pub async fn get_actual_rpg<'a, E>(executor: E) -> Result<Option<i32>, Error>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let actual_rpg = sqlx::query!(
        "SELECT id FROM rpg
        WHERE state = 'Active' AND ended_at IS NULL;"
    )
    .fetch_optional(executor)
    .await?;

    Ok(actual_rpg.map(|rpg| rpg.id))
}

impl Member {
    pub async fn build(pool: &PgPool, id: i64) -> Result<Self, Error> {
        let rpg = get_actual_rpg(pool).await?;

        let (balance, inventories, roles, state, club, job) = try_join!(
            Balance::build(pool, id, rpg),
            Inventories::build(pool, id, rpg),
            Roles::build(pool, id, rpg),
            MemberState::load(pool, id, rpg),
            Club::load(pool, id),
            JobModel::load(pool, id)
        )?;

        Ok(Self {
            id,
            balance,
            inventories,
            job,
            roles,
            state,
            club,
        })
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

impl Club {
    pub async fn load(pool: &PgPool, id: i64) -> Result<Option<Self>, Error> {
        let record = sqlx::query!(
            "SELECT c.id, c.name, cm.role_name
            FROM club_member cm JOIN club c
            ON cm.club = c.id
            WHERE cm.member = $1;",
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(match record {
            Some(res) => Some(Self {
                id: res.id,
                name: res.name,
                relation: AgentRelation::load(pool, id).await?,
                role: ClubRole::load(pool, id).await?,
            }),
            None => None,
        })
    }
}

impl ClubRole {
    pub async fn load(pool: &PgPool, id: i64) -> Result<Self, Error> {
        let record = sqlx::query!(
            "SELECT cr.authority_id AS \"authority_id: AuthorityId\", cr.tr_key, cm.agent_range
            FROM club_member cm JOIN club_role cr
            ON cm.club = cr.club AND cm.role_name = cr.tr_key
            WHERE cm.member = $1;",
            id
        )
        .fetch_one(pool)
        .await?;

        Ok(Self {
            id: record.authority_id,
            tr_key: record.tr_key,
            range: record.agent_range,
            item: RoleItem::load(pool, id).await?,
        })
    }
}

impl RoleItem {
    pub async fn load(pool: &PgPool, id: i64) -> Result<Option<Self>, Error> {
        let record = sqlx::query_as!(
            Self,
            "SELECT ci.item_type AS \"item_type: ClubItemType\", ci.item_tr_key
            FROM club_member cm JOIN club_role cr
            ON cm.club = cr.club AND cm.role_name = cr.tr_key
            LEFT JOIN club_role_item ci ON cm.club = ci.club
            AND ci.role_tr_key = cr.tr_key
            WHERE cm.member = $1;",
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

impl NormalRoles {
    pub async fn load(pool: &PgPool, id: i64) -> Result<Self, Error> {
        let record = sqlx::query_as!(
            Self,
            "SELECT roles AS \"roles: Vec<Role>\"
            FROM member WHERE id = $1;",
            id
        )
        .fetch_one(pool)
        .await?;

        Ok(record)
    }
}

impl RpgRoles {
    pub async fn load(pool: &PgPool, id: i64, rpg: Option<i32>) -> Result<Option<RpgRoles>, Error> {
        Ok(match rpg {
            Some(rpg_id) => {
                let record = sqlx::query_as!(
                    Self,
                    "SELECT role AS \"roles: Vec<RpgRole>\"
                    FROM player WHERE player = $1 AND rpg = $2;",
                    id,
                    rpg_id
                )
                .fetch_one(pool)
                .await?;

                Some(record)
            }
            None => None,
        })
    }
}

impl Roles {
    pub async fn build(pool: &PgPool, id: i64, rpg: Option<i32>) -> Result<Self, Error> {
        Ok(Self {
            normal: NormalRoles::load(pool, id).await?,
            rpg: RpgRoles::load(pool, id, rpg).await?,
        })
    }
}

impl NormalInventory {
    pub async fn load(pool: &PgPool, id: i64) -> Result<Self, Error> {
        let record = sqlx::query_as!(
            ItemInventory,
            "SELECT item.id, item.name, inventory.amount, item.usable, item.quality AS \"quality: Quality\", item.item_type AS \"item_type: ItemType\", victim
            FROM normal_inventory inventory
            INNER JOIN normal_item item ON inventory.item = item.id WHERE inventory.member = $1;",
            id
        ).fetch_all(pool)
        .await?;

        let inventory = record
            .iter()
            .map(|item| (item.id, item.clone().into()))
            .collect::<HashMap<i32, InventoryItem>>();

        Ok(NormalInventory { items: inventory })
    }
}

impl RpgInventory {
    pub async fn load(pool: &PgPool, id: i64, rpg: Option<i32>) -> Result<Option<Self>, Error> {
        Ok(match rpg {
            Some(rpg_id) => {
                let record = sqlx::query_as!(
                    RpgItemInventory,
                    "SELECT item.id, item.name, inventory.amount, item.usable, item.quality AS \"quality: Quality\", item.item_type AS \"item_type: ItemType\", item.armor_type AS \"armor_type: ArmorType\", item.tool_type AS \"tool: Tool\", item.two_handed
                    FROM player_inventory inventory
                    INNER JOIN rpg_item item ON inventory.item = item.id WHERE inventory.player = $1 AND inventory.rpg = $2;",
                    id,
                    rpg_id
                ).fetch_all(pool).await?;

                let inventory = record
                    .iter()
                    .map(|item| (item.id, item.clone().into()))
                    .collect::<HashMap<i32, InventoryItem>>();

                Some(RpgInventory { items: inventory })
            }
            None => None,
        })
    }
}

impl Inventories {
    pub async fn build(pool: &PgPool, id: i64, rpg: Option<i32>) -> Result<Self, Error> {
        Ok(Inventories {
            normal: NormalInventory::load(pool, id).await?,
            rpg: RpgInventory::load(pool, id, rpg).await?,
        })
    }
}

impl MemberState {
    pub async fn load(pool: &PgPool, id: i64, rpg: Option<i32>) -> Result<Self, Error> {
        let can_claim_daily = sqlx::query!("SELECT can_claim_daily_reward($1);", id)
            .fetch_one(pool)
            .await?;

        Ok(Self {
            can_claim_daily: can_claim_daily.can_claim_daily_reward.unwrap(),
            in_gamble: false,
            in_rpg: match rpg {
                Some(rpg) => {
                    let record = sqlx::query!(
                        "SELECT playing FROM player
                        WHERE player = $1 AND rpg = $2;",
                        id,
                        rpg
                    )
                    .fetch_optional(pool)
                    .await?;

                    match record {
                        Some(result) => result.playing.unwrap_or(false),
                        None => false,
                    }
                }
                None => false,
            },
        })
    }
}

impl NormalBalance {
    pub async fn load(pool: &PgPool, id: i64) -> Result<Self, Error> {
        let normal = sqlx::query_as!(
            Self,
            "SELECT balance as yn, points
            FROM member WHERE id = $1;",
            id
        )
        .fetch_one(pool)
        .await?;

        Ok(normal)
    }
}

impl RpgBalance {
    pub async fn load(pool: &PgPool, id: i64, rpg: Option<i32>) -> Result<Option<Self>, Error> {
        Ok(match rpg {
            Some(rpg) => {
                let record = sqlx::query_as!(
                    Self,
                    "SELECT balance as bios, level, experience as exp
                    FROM player WHERE rpg = $1 AND player = $2;",
                    rpg,
                    id
                )
                .fetch_one(pool)
                .await?;

                Some(record)
            }
            None => None,
        })
    }
}

impl Balance {
    pub async fn build(pool: &PgPool, id: i64, rpg: Option<i32>) -> Result<Self, Error> {
        Ok(Self {
            normal: NormalBalance::load(pool, id).await?,
            rpg: RpgBalance::load(pool, id, rpg).await?,
        })
    }
}

impl From<ItemInventory> for InventoryItem {
    fn from(value: ItemInventory) -> Self {
        Self {
            info: Item {
                id: Some(value.id),
                name: value.name,
                number: None,
                usable: value.usable,
                quality: value.quality,
                item_type: value.item_type,
                tool_type: None,
                armor_type: None,
                two_handed: false,
            },
            amount: value.amount,
        }
    }
}

impl From<RpgItemInventory> for InventoryItem {
    fn from(value: RpgItemInventory) -> Self {
        Self {
            info: Item {
                id: Some(value.id),
                name: value.name,
                number: None,
                usable: value.usable,
                quality: value.quality,
                item_type: value.item_type,
                tool_type: value.tool,
                armor_type: value.armor_type,
                two_handed: value.two_handed,
            },
            amount: value.amount,
        }
    }
}
