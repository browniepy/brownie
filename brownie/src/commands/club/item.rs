use crate::{translate, Context, Error, Helper};

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "club",
    subcommands("create", "rename", "delete"),
    subcommand_required
)]
pub async fn item(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "club"
)]
pub async fn create(ctx: Context<'_>, role: String, item: String) -> Result<(), Error> {
    let data = ctx.data();

    let memberr = crate::get_member(ctx, ctx.author().id).await?;
    let member_read = memberr.read().await;

    if member_read.club_id.is_none() {
        let content = translate!(ctx, "member-not-in-club");
        return Err(content.into());
    }

    let club = Helper::get_club(ctx, member_read.club_id.unwrap()).await?;
    let mut club_write = club.write().await;

    club_write.create_item(&data.pool, role, item).await?;

    let content = translate!(ctx, "item-created");
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
pub async fn delete(ctx: Context<'_>, role: String) -> Result<(), Error> {
    let data = ctx.data();

    let memberr = crate::get_member(ctx, ctx.author().id).await?;
    let member_read = memberr.read().await;

    if member_read.club_id.is_none() {
        let content = translate!(ctx, "member-not-in-club");
        return Err(content.into());
    }

    let club = Helper::get_club(ctx, member_read.club_id.unwrap()).await?;
    let mut club_write = club.write().await;

    club_write.delete_item(&data.pool, role).await?;

    let content = translate!(ctx, "item-deleted");
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

    let memberr = crate::get_member(ctx, ctx.author().id).await?;
    let member_read = memberr.read().await;

    if member_read.club_id.is_none() {
        let content = translate!(ctx, "member-not-in-club");
        return Err(content.into());
    }

    let club = Helper::get_club(ctx, member_read.club_id.unwrap()).await?;
    let mut club_write = club.write().await;

    club_write.rename_item(&data.pool, role, name).await?;

    let content = translate!(ctx, "item-renamed");
    ctx.reply(content).await?;

    Ok(())
}
