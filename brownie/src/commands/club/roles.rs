use crate::{translate, Context, Error, Helper};
use poise::serenity_prelude::UserId;

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "club",
    subcommands("create", "rename", "limit", "give"),
    subcommand_required
)]
pub async fn role(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "club"
)]
pub async fn give(ctx: Context<'_>, member: UserId, role: String) -> Result<(), Error> {
    let data = ctx.data();

    let memberr = crate::get_member(ctx, ctx.author().id).await?;
    let member_read = memberr.read().await;

    if member_read.club_id.is_none() {
        let content = translate!(ctx, "member-not-in-club");
        return Err(content.into());
    }

    let club = Helper::get_club(ctx, member_read.club_id.unwrap()).await?;
    let mut club_write = club.write().await;

    club_write
        .change_role(
            &data.pool,
            member.into(),
            role,
            Some(ctx.author().id.into()),
        )
        .await?;

    let content = translate!(ctx, "role-given");
    ctx.reply(content).await?;

    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "club"
)]
pub async fn limit(ctx: Context<'_>, role: String, limit: i32) -> Result<(), Error> {
    let data = ctx.data();

    let member = crate::get_member(ctx, ctx.author().id).await?;
    let member_read = member.read().await;

    if member_read.club_id.is_none() {
        let content = translate!(ctx, "member-not-in-club");
        return Err(content.into());
    }

    let club = Helper::get_club(ctx, member_read.club_id.unwrap()).await?;
    let mut club_write = club.write().await;

    club_write.role_set_limit(&data.pool, role, limit).await?;

    let content = translate!(ctx, "role-limit-set");
    ctx.reply(content).await?;

    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "club"
)]
pub async fn rename(ctx: Context<'_>, role: String, name: String) -> Result<(), Error> {
    let data = ctx.data();

    let member = crate::get_member(ctx, ctx.author().id).await?;
    let member_read = member.read().await;

    if member_read.club_id.is_none() {
        let content = translate!(ctx, "member-not-in-club");
        return Err(content.into());
    }

    let club = Helper::get_club(ctx, member_read.club_id.unwrap()).await?;
    let mut club_write = club.write().await;

    club_write.rename_role(&data.pool, role, name).await?;

    let content = translate!(ctx, "role-renamed");
    ctx.reply(content).await?;

    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "club"
)]
pub async fn create(ctx: Context<'_>, name: String, limit: i32) -> Result<(), Error> {
    let data = ctx.data();

    let member = crate::get_member(ctx, ctx.author().id).await?;
    let member_read = member.read().await;

    if member_read.club_id.is_none() {
        let content = translate!(ctx, "member-not-in-club");
        return Err(content.into());
    }

    let club = Helper::get_club(ctx, member_read.club_id.unwrap()).await?;
    let mut club_write = club.write().await;

    club_write.create_role(&data.pool, name, 50, limit).await?;

    let content = translate!(ctx, "role-created");
    ctx.reply(content).await?;

    Ok(())
}
