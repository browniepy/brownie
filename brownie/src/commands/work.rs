use super::messages::work;
use crate::{Context, Error};

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "economy"
)]
pub async fn work(ctx: Context<'_>) -> Result<(), Error> {
    let id = ctx.author().id;
    let data = ctx.data();

    let member = data.members.get(id.as_ref()).await.unwrap();
    let mut write = member.write().await;

    let earned = write.work(&data.pool).await?;

    work::work(ctx, earned).await?;

    Ok(())
}
