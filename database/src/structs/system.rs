use sqlx::PgPool;

use crate::{
    models::{ItemShop, JobModel, LbMember, Role},
    ErrorT,
};

#[derive(Clone, Debug)]
pub struct System {
    pub top_money: Vec<LbMember>,
    pub top_level: Vec<LbMember>,
    pub shop: Vec<ItemShop>,
    pub jobs: Vec<JobModel>,
}

impl System {
    pub async fn new(pool: &PgPool) -> Self {
        // top 10 money
        let top_money = sqlx::query_as!(
            LbMember,
            "SELECT id, balance, points, level FROM member ORDER BY balance DESC LIMIT 10"
        )
        .fetch_all(pool)
        .await
        .unwrap();

        // top 10 points
        let top_level = sqlx::query_as!(
            LbMember,
            "SELECT id, balance, points, level FROM member ORDER BY points DESC LIMIT 10"
        )
        .fetch_all(pool)
        .await
        .unwrap();

        let jobs = sqlx::query_as!(
            JobModel,
            "SELECT name, description, salary_range, required_role AS \"required_role: Role \", required_level, cooldown FROM job;"
        ).fetch_all(pool).await.unwrap();

        // select id, name, price, description from store table
        let shop = sqlx::query_as!(
            ItemShop,
            "SELECT i.id, i.name, s.price, s.description
            FROM item i JOIN
            store s ON i.id = s.item;"
        )
        .fetch_all(pool)
        .await
        .unwrap();

        Self {
            top_money,
            top_level,
            shop,
            jobs,
        }
    }

    // get shop by desc price
    pub async fn get_shop_desc(&self) -> Vec<ItemShop> {
        let mut shop = self.shop.clone();
        shop.sort_by(|a, b| b.price.cmp(&a.price));
        shop
    }

    // get shop by asc price
    pub async fn get_shop_asc(&self) -> Vec<ItemShop> {
        let mut shop = self.shop.clone();
        shop.sort_by(|a, b| a.price.cmp(&b.price));
        shop
    }

    pub async fn create_job(
        &mut self,
        pool: &PgPool,
        name: String,
        description: Option<String>,
        salary_range: Vec<i32>,
        required_level: i32,
        cooldown: i32,
    ) -> Result<(), ErrorT> {
        sqlx::query!(
            "INSERT INTO job (name, description, salary_range, required_level, cooldown) VALUES ($1, $2, $3, $4, $5)",
            name, description, &salary_range, required_level, cooldown)
            .execute(pool).await?;

        self.jobs.push(JobModel {
            name,
            description,
            salary_range: Some(salary_range),
            required_role: None,
            required_level,
            cooldown,
        });

        Ok(())
    }
}

pub async fn create_item(
    pool: &PgPool,
    name: String,
    description: Option<String>,
) -> Result<i32, ErrorT> {
    let result = sqlx::query!(
        "INSERT INTO item (name, description)
    VALUES ($1, $2) RETURNING id;",
        name,
        description
    )
    .fetch_one(pool)
    .await?;

    Ok(result.id)
}

pub async fn add_item_to_shop(pool: &PgPool, item_id: i32, price: i32) -> Result<(), ErrorT> {
    sqlx::query!(
        "INSERT INTO store (item, price) VALUES ($1, $2);",
        item_id,
        price
    )
    .execute(pool)
    .await?;

    Ok(())
}
