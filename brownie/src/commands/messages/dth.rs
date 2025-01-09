use inflector::Inflector;
use poise::CreateReply;

use super::{Context, Error};
use crate::{
    serenity::*,
    translation::translate,
    types::{handk::*, DropRole},
};

pub async fn foreign_inter(ctx: Context<'_>, inter: &ComponentInteraction) -> Result<(), Error> {
    let embed = CreateEmbed::new().description(translate!(ctx, "wrong-inter"));

    inter
        .create_response(
            ctx,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .embed(embed)
                    .ephemeral(true),
            ),
        )
        .await?;

    Ok(())
}

pub async fn stats(
    ctx: Context<'_>,
    inter: &ComponentInteraction,
    game: &Dth,
) -> Result<(), Error> {
    let checker = game.get_player(DropRole::Checker);
    let dropper = game.get_player(DropRole::Dropper);

    let embed = CreateEmbed::new().description(translate!(
        ctx, "dh-stats",
        checker: &checker.name,
        checkerWasted: checker.wasted_time,
        checkerDeath: checker.death_time,
        dropper: &dropper.name,
        dropperWasted: dropper.wasted_time,
        dropperDeath: dropper.death_time
    ));

    inter
        .create_response(
            ctx,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .embed(embed)
                    .components(vec![CreateActionRow::Buttons(vec![CreateButton::new(
                        format!("{}_stats", ctx.id()),
                    )
                    .style(ButtonStyle::Secondary)
                    .label(translate!(ctx, "stats-btn"))
                    .disabled(true)])]),
            ),
        )
        .await?;

    Ok(())
}

pub async fn check_success_msg(
    ctx: Context<'_>,
    inter: &mut Message,
    checker: &str,
    seconds: u64,
) -> Result<(), Error> {
    let embed = CreateEmbed::new()
        .title(format!("{} acertó", checker))
        .description(translate!(ctx, "dh-round-end-cok", checker: checker, seconds: seconds));

    inter
        .edit(
            ctx,
            EditMessage::new()
                .embed(embed)
                .components(vec![CreateActionRow::Buttons(vec![CreateButton::new(
                    format!("{}_stats", ctx.id()),
                )
                .style(ButtonStyle::Secondary)
                .label(translate!(ctx, "stats-btn"))])]),
        )
        .await?;

    Ok(())
}

pub async fn check_success(
    ctx: Context<'_>,
    inter: &ComponentInteraction,
    checker: &str,
    seconds: u64,
) -> Result<(), Error> {
    let embed = CreateEmbed::new()
        .title(format!("{} acertó", checker))
        .description(translate!(ctx, "dh-round-end-cok", checker: checker, seconds: seconds));

    inter
        .create_response(
            ctx,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .embed(embed)
                    .components(vec![CreateActionRow::Buttons(vec![CreateButton::new(
                        format!("{}_stats", ctx.id()),
                    )
                    .style(ButtonStyle::Secondary)
                    .label(translate!(ctx, "stats-btn"))])]),
            ),
        )
        .await?;

    Ok(())
}

pub async fn already_drop(ctx: Context<'_>, inter: &ComponentInteraction) -> Result<(), Error> {
    inter
        .create_response(
            ctx,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(translate!(ctx, "already-dropped"))
                    .ephemeral(true),
            ),
        )
        .await?;

    Ok(())
}

pub async fn game_accepted(
    ctx: Context<'_>,
    inter: &ComponentInteraction,
    date: &str,
) -> Result<(), Error> {
    let embed = CreateEmbed::new()
        .title("Juego aceptado")
        .description(translate!(ctx, "dh-start", time: date));

    inter
        .create_response(
            ctx,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .embed(embed)
                    .components(vec![CreateActionRow::Buttons(vec![CreateButton::new(
                        format!("{}_stats", ctx.id()),
                    )
                    .style(ButtonStyle::Secondary)
                    .label(translate!(ctx, "stats-btn"))])]),
            ),
        )
        .await?;

    Ok(())
}

pub async fn already_accept(ctx: Context<'_>, inter: &ComponentInteraction) -> Result<(), Error> {
    inter
        .create_response(
            ctx,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("ya aceptaste continuar")
                    .ephemeral(true),
            ),
        )
        .await?;

    Ok(())
}

pub async fn continue_accept(ctx: Context<'_>, inter: &ComponentInteraction) -> Result<(), Error> {
    inter
        .create_response(
            ctx,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .content(translate!(ctx, "continue-accepted"))
                    .components(vec![CreateActionRow::Buttons(vec![CreateButton::new(
                        format!("{}_stats", ctx.id()),
                    )
                    .style(ButtonStyle::Secondary)
                    .label(translate!(ctx, "stats-btn"))])]),
            ),
        )
        .await?;

    Ok(())
}

pub async fn round_expired(
    ctx: Context<'_>,
    inter: &ComponentInteraction,
    message_id: MessageId,
) -> Result<(), Error> {
    inter
        .edit_followup(
            ctx,
            message_id,
            CreateInteractionResponseFollowup::new()
                .content(translate!(ctx, "dh-round-expired"))
                .components(vec![CreateActionRow::Buttons(vec![CreateButton::new(
                    format!("{}_continue", ctx.id()),
                )
                .style(ButtonStyle::Secondary)
                .label(translate!(ctx, "continue-btn"))])]),
        )
        .await?;

    Ok(())
}

pub async fn rcp_alive(
    ctx: Context<'_>,
    inter: &ComponentInteraction,
    checker: &str,
) -> Result<(), Error> {
    let embed = CreateEmbed::new()
        .title("Preparando ronda")
        .description(translate!(ctx, "dh-round-fail-alive", checker: checker));

    inter
        .edit_followup(
            ctx,
            inter.message.id,
            CreateInteractionResponseFollowup::new()
                .embed(embed)
                .components(vec![CreateActionRow::Buttons(vec![CreateButton::new(
                    format!("{}_stats", ctx.id()),
                )
                .style(ButtonStyle::Secondary)
                .label(translate!(ctx, "stats-btn"))])]),
        )
        .await?;

    Ok(())
}

pub async fn rcp_death(
    ctx: Context<'_>,
    inter: &ComponentInteraction,
    checker: &str,
) -> Result<(), Error> {
    let embed = CreateEmbed::new()
        .title("Final")
        .description(translate!(ctx, "dh-round-fail-death", checker: checker));

    inter
        .edit_followup(
            ctx,
            inter.message.id,
            CreateInteractionResponseFollowup::new().embed(embed),
        )
        .await?;

    Ok(())
}

pub async fn reanimation(
    ctx: Context<'_>,
    inter: &ComponentInteraction,
    game: &Dth,
) -> Result<(), Error> {
    let checker = game.get_player(DropRole::Checker);

    let embed = CreateEmbed::new()
        .title(format!("{} falló", checker.name))
        .description(translate!(ctx, "dh-try-reanimate", checker: &checker.name));

    inter
        .edit_followup(
            ctx,
            inter.message.id,
            CreateInteractionResponseFollowup::new()
                .embed(embed)
                .components(vec![]),
        )
        .await?;

    Ok(())
}

pub async fn start_round(
    ctx: Context<'_>,
    inter: &ComponentInteraction,
    game: &Dth,
) -> Result<Message, Error> {
    let checker = game.get_player(DropRole::Checker);
    let dropper = game.get_player(DropRole::Dropper);

    let embed = CreateEmbed::new()
        .title("Ronda iniciada")
        .description(translate!(ctx, "dh-inround", checker: &checker.name, dropper: &dropper.name));

    let message = inter
        .create_followup(
            ctx,
            CreateInteractionResponseFollowup::new()
                .embed(embed)
                .components(vec![CreateActionRow::Buttons(vec![
                    CreateButton::new(format!("{}_check", ctx.id()))
                        .style(ButtonStyle::Secondary)
                        .label(translate!(ctx, "dh-check-btn")),
                    CreateButton::new(format!("{}_drop", ctx.id()))
                        .style(ButtonStyle::Secondary)
                        .label(translate!(ctx, "dh-drop-btn")),
                ])]),
        )
        .await?;

    Ok(message)
}

pub async fn proposal(ctx: Context<'_>, name: &str) -> Result<Message, Error> {
    let embed = CreateEmbed::new()
        .description(translate!(ctx, "dh-gamble-proposal", user: name.to_title_case()));

    let message = ctx
        .send(
            CreateReply::default()
                .embed(embed)
                .components(vec![CreateActionRow::Buttons(vec![CreateButton::new(
                    format!("{}_accept", ctx.id()),
                )
                .style(ButtonStyle::Secondary)
                .label(translate!(ctx, "accept-btn"))])]),
        )
        .await?
        .into_message()
        .await?;

    Ok(message)
}
