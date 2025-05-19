use crate::{get_guild, translate, Context, Data, Error};
use poise::{
    serenity_prelude::{CreateAllowedMentions, CreateMessage, GuildChannel, GuildId, Member},
    CreateReply,
};

#[poise::command(slash_command, subcommands("message", "toggle", "channel", "send"))]
pub async fn greet(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command)]
async fn message(ctx: Context<'_>, config: String) -> Result<(), Error> {
    let data = ctx.data();

    let guild = get_guild(data, ctx.guild_id().unwrap()).await?;
    let mut write = guild.write().await;

    write.greeting.update_message(&data.pool, config).await?;

    let content = translate!(ctx, "greet-updated");
    ctx.reply(content).await?;

    Ok(())
}

#[poise::command(slash_command)]
async fn toggle(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();

    let guild = get_guild(data, ctx.guild_id().unwrap()).await?;
    let mut write = guild.write().await;

    write.toggle_greeting(&data.pool).await?;

    let content = match write.greeting.enabled {
        true => translate!(ctx, "greet-enabled"),
        false => translate!(ctx, "greet-disabled"),
    };

    ctx.reply(content).await?;

    Ok(())
}

#[poise::command(slash_command)]
async fn channel(ctx: Context<'_>, channel: Option<GuildChannel>) -> Result<(), Error> {
    let data = ctx.data();

    let guild = get_guild(data, ctx.guild_id().unwrap()).await?;
    let mut write = guild.write().await;

    let channel = channel.map(|channel| channel.id.into());
    write.greeting.set_channel(&data.pool, channel).await?;

    let content = match channel {
        Some(_) => translate!(ctx, "greet-channel-set"),
        None => translate!(ctx, "greet-channel-removed"),
    };

    ctx.reply(content).await?;

    Ok(())
}

#[poise::command(slash_command)]
async fn send(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();

    let greet = get_greet(
        data,
        ctx.guild_id().unwrap(),
        ctx.author_member().await.unwrap().as_ref(),
    )
    .await?;

    ctx.send(greet).await?;

    Ok(())
}

async fn get_greet(data: &Data, id: GuildId, member: &Member) -> Result<CreateReply, Error> {
    let guild = get_guild(data, id).await?;
    let read = guild.read().await;

    let mentions = match read.greeting.mention {
        true => CreateAllowedMentions::default(),
        false => crate::mentions(),
    };

    let builder = read
        .greeting
        .get_greet_reply(member, 137, "Server")?
        .allowed_mentions(mentions);

    Ok(builder)
}
