use super::{
    autocomplete::items_auto,
    check::{self_can_gamble, user_can_gamble},
};
use crate::{get_member, translate, Context, Error, Parser};
use poise::serenity_prelude::User;

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "economy",
    subcommand_required,
    subcommands("item", "balance")
)]
pub async fn give(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "economy",
    check = "self_can_gamble"
)]
pub async fn balance(ctx: Context<'_>, user: User, amount: String) -> Result<(), Error> {
    user_can_gamble(ctx, user.clone()).await?;

    let bios = Parser::amount(ctx, ctx.author().id, Some(amount), 500).await?;

    let author = get_member(ctx, ctx.author().id).await?;
    let mut author_write = author.write().await;

    if author_write.get_bios() < bios {
        let err = translate!(ctx, "not-enough-bios");
        return Err(err.into());
    }

    let receiver = get_member(ctx, user.id).await?;
    let mut receiver_write = receiver.write().await;

    let data = ctx.data();
    let mut tx = data.pool.begin().await?;

    if author_write.decrease_bios(&mut *tx, bios).await.is_err() {
        tx.rollback().await?;

        let err = translate!(ctx, "transaction-error");
        return Err(err.into());
    }

    if receiver_write.increase_bios(&mut *tx, bios).await.is_err() {
        tx.rollback().await?;

        let err = translate!(ctx, "transaction-error");
        return Err(err.into());
    }

    tx.commit().await?;

    let bios_with_commas = Parser::num_with_commas(bios);
    let content = translate!(ctx, "bios-give", bios: bios_with_commas);
    ctx.reply(content).await?;

    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "economy",
    check = "self_can_gamble"
)]
pub async fn item(
    ctx: Context<'_>,
    user: User,
    #[autocomplete = "items_auto"] item: String,
    amount: i32,
) -> Result<(), Error> {
    user_can_gamble(ctx, user.clone()).await?;

    let item_id = item
        .split_whitespace()
        .last()
        .unwrap()
        .parse::<i32>()
        .unwrap();

    let author = get_member(ctx, ctx.author().id).await?;
    let mut author_write = author.write().await;

    match author_write.get_item_by_id(item_id) {
        Some(item) => {
            if item.amount < amount {
                let err = translate!(ctx, "not-enough-items");
                return Err(err.into());
            }
        }
        None => {
            let err = translate!(ctx, "item-not-found");
            return Err(err.into());
        }
    }

    let receiver = get_member(ctx, user.id).await?;
    let mut receiver_write = receiver.write().await;

    let data = ctx.data();
    let mut tx = data.pool.begin().await?;

    match author_write.remove_item(&mut *tx, item_id, amount).await? {
        Some(removed_item) => {
            if receiver_write
                .add_item(&mut *tx, removed_item.into(), amount)
                .await
                .is_err()
            {
                tx.rollback().await?;

                let err = translate!(ctx, "transaction-error");
                return Err(err.into());
            }
        }
        None => {
            tx.rollback().await?;

            let err = translate!(ctx, "transaction-error");
            return Err(err.into());
        }
    }

    tx.commit().await?;

    let content = translate!(ctx, "item-give", amount: amount, item: item);
    ctx.reply(content).await?;

    Ok(())
}
