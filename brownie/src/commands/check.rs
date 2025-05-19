use poise::serenity_prelude::{Mentionable, User};

use super::{translate, Context, Error};
use crate::get_member;

pub async fn self_can_gamble(ctx: Context<'_>) -> Result<bool, Error> {
    let member = get_member(ctx, ctx.author().id).await?;
    let member_read = member.read().await;

    if member_read.state.in_gamble {
        let content = translate!(ctx, "self-in-gamble");
        return Err(content.into());
    }

    Ok(!member_read.state.in_gamble)
}

pub async fn while_in_bet(ctx: Context<'_>) -> Result<bool, Error> {
    let member = get_member(ctx, ctx.author().id).await?;
    let member_read = member.read().await;

    if member_read.state.in_gamble {
        let content = translate!(ctx, "while-in-gamble");
        return Err(content.into());
    }

    Ok(!member_read.state.in_gamble)
}

pub async fn user_can_gamble(ctx: Context<'_>, user: User) -> Result<(), Error> {
    let member = get_member(ctx, user.id).await?;
    let member_read = member.read().await;

    if member_read.state.in_gamble {
        let name = user.mention().to_string();
        let content = translate!(ctx, "user-in-gamble", user: name);
        return Err(content.into());
    }

    Ok(())
}
