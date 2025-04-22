mod connect;
pub mod models;
pub mod structs;

#[cfg(test)]
mod tests;

pub use connect::connect;
use models::{ItemInventory, RpgItemInventory};
pub use moka::future::{Cache, CacheBuilder};
pub use sqlx::{query, PgPool};
use structs::{Item, Member, System};
use tokio::sync::Mutex;

pub use std::sync::Arc;
pub use tokio::sync::RwLock;

pub mod items;

pub mod player;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

impl From<Member> for Arc<RwLock<Member>> {
    fn from(member: Member) -> Self {
        Arc::new(RwLock::new(member))
    }
}

impl From<System> for Arc<Mutex<System>> {
    fn from(system: System) -> Self {
        Arc::new(Mutex::new(system))
    }
}

impl From<ItemInventory> for Item {
    fn from(value: ItemInventory) -> Self {
        Self {
            id: Some(value.id),
            name: value.name,
            number: None,
            usable: value.usable,
            item_type: value.item_type,
            quality: value.quality,
            tool_type: None,
            armor_type: None,
            two_handed: false,
        }
    }
}

impl From<RpgItemInventory> for Item {
    fn from(value: RpgItemInventory) -> Self {
        Self {
            id: Some(value.id),
            name: value.name,
            number: None,
            usable: value.usable,
            item_type: value.item_type,
            quality: value.quality,
            tool_type: None,
            armor_type: None,
            two_handed: value.two_handed,
        }
    }
}
