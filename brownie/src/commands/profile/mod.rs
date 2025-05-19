mod inventory;
pub use inventory::inventory;

use super::{translate, Context, Error};
use crate::{get_member, serenity::User, Parser};
use inflector::Inflector;

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "economy"
)]
pub async fn profile(ctx: Context<'_>, user: Option<User>) -> Result<(), Error> {
    let user_id = user.clone().unwrap_or(ctx.author().clone()).id;

    let member = get_member(ctx, user_id).await?;
    let read = member.read().await;

    Err("unimplemented".into())
}

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "economy"
)]
pub async fn points(ctx: Context<'_>, user: Option<User>) -> Result<(), Error> {
    let user_unwrap = user.clone().unwrap_or(ctx.author().clone());

    let member = get_member(ctx, user_unwrap.id).await?;
    let read = member.read().await;

    let points = Parser::num_with_commas(read.get_points() as i64);

    let content = match user {
        None => translate!(ctx, "points-self", points: points),
        Some(user) => {
            let name = user.display_name().to_title_case();
            translate!(ctx, "points-other", user: name, points: points)
        }
    };

    ctx.reply(content).await?;
    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "economy"
)]
pub async fn balance(ctx: Context<'_>, user: Option<User>) -> Result<(), Error> {
    let user_unwrap = user.clone().unwrap_or(ctx.author().clone());
    let id = user_unwrap.id;

    let member = get_member(ctx, id).await?;
    let read = member.read().await;

    let balance = Parser::num_with_commas(read.get_bios());

    let content = match user {
        None => translate!(ctx, "balance-self", bios: balance),
        Some(user) => {
            let name = user.display_name().to_title_case();
            translate!(ctx, "balance-other", user: name, bios: balance)
        }
    };

    ctx.reply(content).await?;
    Ok(())
}
