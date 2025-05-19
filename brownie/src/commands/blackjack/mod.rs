mod response;

use crate::{Context, Duration, Error, Parser};
use poise::serenity_prelude::ComponentInteractionCollector;
use types::blackjack::Blackjack;

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn blackjack(ctx: Context<'_>, amount: Option<String>) -> Result<(), Error> {
    let bet = Parser::amount(ctx, ctx.author().id, amount, 500).await?;

    let mut blackjack = Blackjack::new(ctx.author().clone(), bet);

    {
        let member = crate::get_member(ctx, ctx.author().id).await?;
        let mut write = member.write().await;

        blackjack.deal_cards(&mut write.deck);
    }

    while let Some(inter) = ComponentInteractionCollector::new(ctx)
        .timeout(Duration::from_secs(60))
        .await
    {
        if inter.user.id != ctx.author().id {
            continue;
        }

        if inter.data.custom_id == format!("hit_{}", ctx.id()) {
            let member = crate::get_member(ctx, ctx.author().id).await?;
            let mut write = member.write().await;

            blackjack.player_hit(&mut write.deck);
        }

        if inter.data.custom_id == format!("stand_{}", ctx.id()) {
            let member = crate::get_member(ctx, ctx.author().id).await?;
            let mut write = member.write().await;

            while blackjack.dealer.hand_value(false) < 17 {
                blackjack.dealer_hit(&mut write.deck);
            }
        }

        if inter.data.custom_id == format!("double_{}", ctx.id()) {
            let member = crate::get_member(ctx, ctx.author().id).await?;
            let mut write = member.write().await;

            blackjack.player_hit(&mut write.deck);
        }
    }

    Ok(())
}
