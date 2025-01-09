mod connect;
pub mod models;
pub mod structs;
pub mod types;

#[cfg(test)]
mod tests;

pub use connect::connect;
pub use moka::future::{Cache, CacheBuilder};
pub use sqlx::{query, PgPool};
use structs::Member;
pub use types::{Card, ErrorT, Pale};

pub use std::sync::Arc;
pub use tokio::sync::RwLock;

impl From<Member> for Arc<RwLock<Member>> {
    fn from(member: Member) -> Self {
        Arc::new(RwLock::new(member))
    }
}
