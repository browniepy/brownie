use crate::{ErrorT, PgPool};
use sqlx::postgres::PgPoolOptions;

pub async fn connect() -> Result<PgPool, ErrorT> {
    let url = std::env::var("pgserver").unwrap();
    let pool = PgPoolOptions::new().connect(url.as_str()).await?;
    Ok(pool)
}
