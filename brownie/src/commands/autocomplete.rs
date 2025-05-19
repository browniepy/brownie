use crate::{helpers, translate, Context};
use futures::{Stream, StreamExt};

pub async fn items_auto<'a>(ctx: Context<'_>, partial: &'a str) -> impl Stream<Item = String> + 'a {
    let player = helpers::get_member(ctx, ctx.author().id).await.unwrap();
    let read = player.read().await;

    let mut items = read
        .get_inventory()
        .iter()
        .map(|item| {
            let tr_item = translate!(ctx, &item.info.name.clone());
            format!(
                "{} {} id {}",
                item.amount,
                tr_item,
                item.info.id.unwrap_or_default()
            )
        })
        .collect::<Vec<String>>();

    if items.is_empty() {
        items.push(translate!(ctx, "empty-inventory"));
    }

    futures::stream::iter(items)
        .filter(move |name| {
            futures::future::ready(name.to_lowercase().contains(&partial.to_lowercase()))
        })
        .map(|name| name.to_string())
}

pub async fn dices_auto<'a>(
    _ctx: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    let choices = vec!["Pair", "Unpair"];

    futures::stream::iter(choices)
        .filter(move |name| {
            futures::future::ready(name.to_lowercase().contains(&partial.to_lowercase()))
        })
        .map(|name| name.to_string())
}
