use crate::{get_system, translate, Context, Error};

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "economy",
    subcommand_required,
    subcommands("view", "buy")
)]
pub async fn shop(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "economy"
)]
pub async fn view(ctx: Context<'_>) -> Result<(), Error> {
    let system = get_system(ctx).await;
    let lock = system.lock().await;

    let products = lock
        .shop
        .iter()
        .map(|product| crate::PageField {
            title: translate!(ctx, &product.item.name),
            description: translate!(ctx, "product-info",
                price: product.price),
        })
        .collect::<Vec<_>>();

    crate::paginate(ctx, products).await?;

    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "economy"
)]
pub async fn buy(ctx: Context<'_>, item: i32) -> Result<(), Error> {
    Ok(())
}
