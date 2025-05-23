use crate::{translate, Context, Error, Helper};
use database::{models::ClubType, structs::club::Club};
use poise::serenity_prelude::UserId;

mod roles;
use roles::role;

mod item;
use item::item;

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    subcommand_required,
    subcommands(
        "create",
        "rename",
        "description",
        "kind",
        "transfer",
        "role",
        "item",
        "leave"
    ),
    category = "club"
)]
pub async fn club(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "club"
)]
pub async fn leave(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();

    let member = crate::get_member(ctx, ctx.author().id).await?;
    let mut member_write = member.write().await;

    if member_write.club_id.is_none() {
        let content = translate!(ctx, "user-not-in-club");
        return Err(content.into());
    }

    let club = Helper::get_club(ctx, member_write.club_id.unwrap()).await?;
    let mut club_write = club.write().await;

    club_write.kick_member(&data.pool, member_write.id).await?;

    member_write.club_id = None;

    let content = translate!(ctx, "club-left");
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
pub async fn create(ctx: Context<'_>, name: String) -> Result<(), Error> {
    let data = ctx.data();

    let member = crate::get_member(ctx, ctx.author().id).await?;
    let mut member_write = member.write().await;

    if member_write.club_id.is_some() {
        let content = translate!(ctx, "member-cant-create-club");
        return Err(content.into());
    }

    let club = Club::create(&data.pool, member_write.id, name).await?;

    member_write.club_id = Some(club.id);
    Helper::cache_club(ctx, club.id).await;

    let content = translate!(ctx, "club-created");
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
pub async fn rename(ctx: Context<'_>, name: String) -> Result<(), Error> {
    let data = ctx.data();

    let member = crate::get_member(ctx, ctx.author().id).await?;
    let member_read = member.read().await;

    if member_read.club_id.is_none() {
        let content = translate!(ctx, "user-not-in-club");
        return Err(content.into());
    }

    let club = Helper::get_club(ctx, member_read.club_id.unwrap()).await?;
    let mut club_write = club.write().await;

    if club_write.can_manage_club(member_read.id) {
        club_write.rename(&data.pool, name).await?;

        let content = translate!(ctx, "club-renamed");
        ctx.reply(content).await?;

        Ok(())
    } else {
        let content = translate!(ctx, "member-cant-manage-club");
        Err(content.into())
    }
}

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "club"
)]
pub async fn description(ctx: Context<'_>, description: Option<String>) -> Result<(), Error> {
    let data = ctx.data();

    let member = crate::get_member(ctx, ctx.author().id).await?;
    let member_read = member.read().await;

    if member_read.club_id.is_none() {
        let content = translate!(ctx, "user-not-in-club");
        return Err(content.into());
    }

    let club = Helper::get_club(ctx, member_read.club_id.unwrap()).await?;
    let mut club_write = club.write().await;

    if club_write.can_manage_club(member_read.id) {
        club_write
            .set_description(&data.pool, description)
            .await
            .unwrap();

        let content = translate!(ctx, "club-description-set");
        ctx.reply(content).await?;

        Ok(())
    } else {
        let content = translate!(ctx, "member-cant-manage-club");
        Err(content.into())
    }
}

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "club"
)]
pub async fn kind(ctx: Context<'_>, kind: ClubType) -> Result<(), Error> {
    let data = ctx.data();

    let member = crate::get_member(ctx, ctx.author().id).await?;
    let member_read = member.read().await;

    if member_read.club_id.is_none() {
        let content = translate!(ctx, "user-not-in-club");
        return Err(content.into());
    }

    let club = Helper::get_club(ctx, member_read.club_id.unwrap()).await?;
    let mut club_write = club.write().await;

    if !club_write.can_manage_club(member_read.id) {
        let content = translate!(ctx, "member-cant-manage-club");
        return Err(content.into());
    }

    if club_write.set_type(&data.pool, kind).await.is_err() {
        let content = translate!(ctx, "unkown-error");
        return Err(content.into());
    }

    let content = translate!(ctx, "club-type-set");
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
pub async fn transfer(ctx: Context<'_>, user: UserId) -> Result<(), Error> {
    let data = ctx.data();

    let member = crate::get_member(ctx, ctx.author().id).await?;
    let member_read = member.read().await;

    if member_read.club_id.is_none() {
        let content = translate!(ctx, "user-not-in-club");
        return Err(content.into());
    }

    let club = Helper::get_club(ctx, member_read.club_id.unwrap()).await?;
    let mut club_write = club.write().await;

    if !club_write.is_leader(member_read.id) {
        let content = translate!(ctx, "only-leader-can-transfer");
        return Err(content.into());
    }

    club_write.transfer(&data.pool, user.into()).await?;

    let content = translate!(ctx, "club-transfered");
    ctx.reply(content).await?;

    Ok(())
}
