use ::types::Rooms;
pub use database::{structs::Member, Cache, PgPool};
pub use poise::serenity_prelude as serenity;
use serenity::UserId;

pub use std::{sync::Arc, time::Duration};
use tokio::sync::RwLock;

pub mod translation;
pub use translation::{read_ftl, Translations};

mod client;
pub mod commands;
pub mod mpsc_data;
pub mod types;

mod parser;
pub use parser::Parse;

pub struct Data {
    pub pool: PgPool,
    pub members: Cache<UserId, Arc<RwLock<Member>>>,
    pub translations: Translations,
    pub rooms: Rooms,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    ::tracing_subscriber::fmt::init();

    let mut client = client::build().await?;
    client.start().await.map_err(Into::into)
}

pub async fn cache(ctx: Context<'_>, id: UserId) {
    let data = ctx.data();

    data.members
        .entry_by_ref(id.as_ref())
        .or_insert_with(async {
            let user = id.to_user(ctx).await.unwrap();
            tracing::info!("cached {}", user.name);

            Member::builder(id.into())
                .build(&data.pool)
                .await
                .unwrap()
                .into()
        })
        .await;
}

pub async fn refresh_cache(ctx: Context<'_>, id: UserId) {
    let data = ctx.data();

    let entry = data.members.get(id.as_ref()).await;

    if entry.is_some() {
        data.members.remove(id.as_ref()).await;
    }

    cache(ctx, id).await;
}

pub async fn get_member(ctx: Context<'_>, id: UserId) -> Result<Arc<RwLock<Member>>, Error> {
    let data = ctx.data();

    let member_entry = data.members.get(id.as_ref()).await;

    match member_entry {
        Some(member) => Ok(member),
        None => {
            ctx.defer().await?;

            cache(ctx, id).await;

            Ok(data.members.get(id.as_ref()).await.unwrap())
        }
    }
}

#[derive(poise::ChoiceParameter)]
pub enum Game {
    Contradict,
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Game::Contradict => write!(f, "Contradict"),
        }
    }
}
