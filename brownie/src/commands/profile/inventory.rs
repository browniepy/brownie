use super::{Context, Error};
use crate::{get_member, translate, Parser};
use poise::serenity_prelude::User;

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn inventory(ctx: Context<'_>, user: Option<User>) -> Result<(), Error> {
    let user_id = user.clone().unwrap_or(ctx.author().clone()).id;
    let member = get_member(ctx, user_id).await?;
    let member_read = member.read().await;

    if member_read.get_inventory().is_empty() {
        let err = translate!(ctx, "empty-inventory");
        return Err(err.into());
    }

    let translated_inventory = member_read
        .get_inventory()
        .iter()
        .map(|item| {
            let name = translate!(ctx, &item.info.name.clone());
            let amount = Parser::num_with_commas(item.amount as i64);
            format!("{} {}", amount, name)
        })
        .collect::<Vec<String>>();

    ctx.say(translated_inventory.join(", ")).await?;
    Ok(())
}
