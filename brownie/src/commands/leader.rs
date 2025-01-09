use database::models::Role;
use poise::{serenity_prelude::User, CreateReply};

use crate::{get_member, Context, Error};

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    subcommands("referee", "member"),
    subcommand_required
)]
pub async fn assign(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn referee(ctx: Context<'_>, user: User) -> Result<(), Error> {
    let a = user.clone();

    let author = get_member(ctx, ctx.author().id).await?;
    let author = author.read().await;

    if !author.roles.contains(&Role::Leader) {
        return Err("Command only for the leader".into());
    }

    let user = get_member(ctx, user.id).await?;
    let mut user = user.write().await;

    let data = ctx.data();
    user.give_role(Role::Referee, &data.pool).await?;

    ctx.send(CreateReply::default().content(format!(
        "¡Ahora {} es el Referí n{}!",
        a.name,
        user.referee_range.unwrap()
    )))
    .await?;

    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn member(ctx: Context<'_>, user: User) -> Result<(), Error> {
    let author = get_member(ctx, ctx.author().id).await?;
    let author = author.read().await;

    if !author.roles.contains(&Role::Leader) {
        return Err("Command only for the leader".into());
    }

    let user = get_member(ctx, user.id).await?;
    let mut user = user.write().await;

    let data = ctx.data();
    user.give_role(Role::Member, &data.pool).await?;

    ctx.say(":v").await?;

    Ok(())
}
