use std::time::Duration;

use moka::future::Cache;
use sqlx::PgPool;

use crate::{
    models::{AuthorityId, BoardMember, ItemType, JobModel, Quality, Role},
    structs::{Item, Product},
    Error,
};

pub struct System {
    pub top_money: Vec<BoardMember>,
    pub top_level: Vec<BoardMember>,
    pub shop: Vec<Product>,
    pub jobs: Vec<JobModel>,
}

impl System {
    pub async fn new(pool: &PgPool) -> Self {
        // top 10 money
        let top_money = sqlx::query_as!(
            BoardMember,
            "SELECT id, balance, points  FROM member ORDER BY balance DESC LIMIT 10"
        )
        .fetch_all(pool)
        .await
        .unwrap();

        // top 10 points
        let top_level = sqlx::query_as!(
            BoardMember,
            "SELECT id, balance, points FROM member ORDER BY points DESC LIMIT 10"
        )
        .fetch_all(pool)
        .await
        .unwrap();

        let jobs = sqlx::query_as!(
            JobModel,
            "SELECT id, name, salary, required_role AS \"required_role: Role \", required_points, cooldown FROM job;"
        ).fetch_all(pool).await.unwrap();

        // select id, name, price, description from store table
        let shop = sqlx::query!(
            "SELECT i.id, i.name, i.usable, i.quality AS \"quality: Quality\", item_type AS \"item_type: ItemType\", s.price
            FROM normal_item i JOIN
            normal_shop s ON i.id = s.item;"
        )
        .fetch_all(pool)
        .await
        .unwrap();

        let shop = shop
            .iter()
            .map(|item| Product {
                item: Item {
                    id: Some(item.id),
                    name: item.name.clone(),
                    number: None,
                    usable: item.usable,
                    item_type: item.item_type.clone(),
                    quality: item.quality.clone(),
                    two_handed: false,
                },
                price: item.price.unwrap_or_default(),
                stock: None,
                description: None,
            })
            .collect::<Vec<Product>>();

        Self {
            top_money,
            top_level,
            shop,
            jobs,
        }
    }

    pub async fn get_job_names(&self) -> Vec<String> {
        self.jobs.iter().map(|job| job.name.clone()).collect()
    }

    // get shop by desc price
    pub async fn get_shop_desc(&self) -> Vec<Product> {
        let mut shop = self.shop.clone();
        shop.sort_by(|a, b| b.price.cmp(&a.price));
        shop
    }

    // get shop by asc price
    pub async fn get_shop_asc(&self) -> Vec<Product> {
        let mut shop = self.shop.clone();
        shop.sort_by(|a, b| a.price.cmp(&b.price));
        shop
    }

    pub async fn get_item_by_id(&self, pool: &PgPool, id: i32) -> Result<Item, Error> {
        let item = sqlx::query!(
            "SELECT id, name, usable, item_type AS \"item_type: ItemType \", quality AS \"quality: Quality \"
            FROM normal_item WHERE id = $1",
            id
        )
        .fetch_one(pool)
        .await?;

        let item = Item {
            id: Some(item.id),
            name: item.name,
            number: None,
            usable: item.usable,
            quality: item.quality,
            item_type: item.item_type,
            two_handed: false,
        };

        Ok(item)
    }

    pub fn paginate_shop(&self, page_size: usize) -> Vec<Vec<Product>> {
        self.shop
            .chunks(page_size)
            .map(|chunk| chunk.to_vec())
            .collect()
    }

    pub fn paginate_jobs(&self, page_size: usize) -> Vec<Vec<JobModel>> {
        self.jobs
            .chunks(page_size)
            .map(|chunk| chunk.to_vec())
            .collect()
    }
}
