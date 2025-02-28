use database::structs::{Member, System};
pub use database::{Cache, PgPool};

use inflector::Inflector;
pub use poise::serenity_prelude as serenity;
use serenity::UserId;

pub use std::{sync::Arc, time::Duration};
pub use tokio::sync::{Mutex, RwLock};

pub mod translation;
pub use translation::*;

mod client;
pub mod commands;

pub mod helpers;
pub use helpers::*;

mod parser;
pub use parser::*;

pub struct Data {
    pub pool: PgPool,
    pub members: Cache<UserId, Arc<RwLock<Member>>>,
    pub translations: Translations,
    pub system: Cache<(), Arc<Mutex<System>>>,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    ::tracing_subscriber::fmt::init();

    let mut client = client::build().await?;
    client.start().await.map_err(Into::into)
}

use futures::{Stream, StreamExt};

pub async fn debts_auto<'a>(ctx: Context<'_>, partial: &'a str) -> impl Stream<Item = String> + 'a {
    let player = helpers::get_member(ctx, ctx.author().id).await.unwrap();
    let read = player.read().await;

    let mut debt_users = Vec::new();

    for id in read.get_debt_users().iter() {
        let name = ctx
            .http()
            .get_user(UserId::new(*id as u64))
            .await
            .unwrap()
            .display_name()
            .to_title_case();
        debt_users.push(name);
    }

    if debt_users.is_empty() {
        debt_users.push(String::from("Empty"))
    }

    futures::stream::iter(debt_users)
        .filter(move |name| {
            futures::future::ready(name.to_lowercase().contains(&partial.to_lowercase()))
        })
        .map(|name| name.to_string())
}

pub async fn items_auto<'a>(ctx: Context<'_>, partial: &'a str) -> impl Stream<Item = String> + 'a {
    let player = helpers::get_member(ctx, ctx.author().id).await.unwrap();
    let read = player.read().await;

    futures::stream::iter(
        read.clone()
            .inventory
            .values()
            .map(|item| item.name.clone().unwrap())
            .collect::<Vec<_>>(),
    )
    .filter(move |name| {
        futures::future::ready(name.to_lowercase().contains(&partial.to_lowercase()))
    })
    .map(|name| name.to_string())
}

#[derive(poise::ChoiceParameter)]
pub enum Game {
    Contradict,
    NimTypeZero,
    BlackJack,
    RussianRoulette,
    Dices,
    Falaris,
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Game::Contradict => write!(f, "Contradict"),
            Game::NimTypeZero => write!(f, "Nim Type Zero"),
            Game::BlackJack => write!(f, "BlackJack"),
            Game::RussianRoulette => write!(f, "Rr"),
            Game::Dices => write!(f, "Dices"),
            Game::Falaris => write!(f, "Falaris"),
        }
    }
}
