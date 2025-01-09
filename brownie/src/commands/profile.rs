use inflector::Inflector;
use poise::CreateReply;

use super::messages::work;
use crate::{get_member, serenity::User, translation::translate, Context, Error};

// commands related to profiles

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "economy"
)]
pub async fn profile(ctx: Context<'_>, user: Option<User>) -> Result<(), Error> {
    let target = user.clone().unwrap_or(ctx.author().to_owned());
    let id = target.id;

    let member = get_member(ctx, id).await?;
    let read = member.read().await;

    work::profile(
        ctx,
        &target.name.to_title_case(),
        read.roles.clone(),
        read.referee_range,
    )
    .await?;

    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "economy"
)]
pub async fn stl(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();

    let member = get_member(ctx, ctx.author().id).await?;
    let read = member.read().await;

    let content = if read.can_stl(&data.pool).await? {
        "Cumples los requisitos para stl"
    } else {
        "No cumples los requisitos para stl"
    };

    ctx.send(CreateReply::default().content(content)).await?;

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
    let id = user.clone().unwrap_or(ctx.author().to_owned()).id;

    let member = get_member(ctx, id).await?;
    let read = member.read().await;

    let content = match user {
        None => translate!(ctx, "balance-self", balance: read.balance),
        Some(user) => {
            let name = user.display_name().to_title_case();
            translate!(ctx, "balance-other", name: name, balance: read.balance)
        }
    };

    ctx.send(CreateReply::default().content(content)).await?;

    Ok(())
}
