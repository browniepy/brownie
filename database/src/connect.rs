use std::time::Duration;

use crate::{Error, PgPool};
use sqlx::postgres::PgPoolOptions;

pub async fn connect() -> Result<PgPool, Error> {
    let url = std::env::var("pgserver").unwrap();
    let pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(2))
        .max_connections(15)
        .connect(url.as_str())
        .await?;
    Ok(pool)
}
