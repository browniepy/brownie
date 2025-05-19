use database::{
    structs::{club::Club, guild::Guild, Member, System},
    Cache, PgPool,
};
pub use poise::serenity_prelude as serenity;
use serenity::{GuildId, UserId};
pub use std::{sync::Arc, time::Duration};
pub use tokio::sync::{Mutex, RwLock};

pub mod translation;
pub use translation::*;

mod client;
pub mod commands;

pub mod helpers;
pub use helpers::*;

mod parser;
pub use parser::Parser;

mod paginator;
pub use paginator::{paginate, PageField};

pub struct Data {
    pub pool: PgPool,
    pub members: Cache<UserId, Arc<RwLock<Member>>>,
    pub guilds: Cache<GuildId, Arc<RwLock<Guild>>>,
    pub translations: Translations,
    pub system: Cache<(), Arc<Mutex<System>>>,
    pub clubs: Cache<i64, Arc<RwLock<Club>>>,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let mut client = client::build().await?;
    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        shard_manager.shutdown_all().await;
    });

    client.start_autosharded().await.map_err(Into::into)
}
