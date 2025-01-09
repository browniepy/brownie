use inflector::Inflector;
use poise::{
    serenity_prelude::{
        ButtonStyle, ComponentInteraction, CreateActionRow, CreateAllowedMentions, CreateButton,
        CreateInteractionResponse, CreateInteractionResponseFollowup,
        CreateInteractionResponseMessage, Message, MessageId, User,
    },
    CreateReply,
};

use crate::{translation::translate, types::zeronim::Nim, Context, Error};

pub async fn game_end(
    ctx: Context<'_>,
    inter: &ComponentInteraction,
    nim: &Nim,
    message: MessageId,
) -> Result<(), Error> {
    let actual = nim.current_player();
    let rival = nim.players.last().unwrap();

    let table = {
        let vec = nim
            .table_cards
            .iter()
            .map(|card| card.name(ctx))
            .collect::<Vec<String>>();
        format!("{:?}", vec)
    };

    inter
        .edit_followup(
            ctx,
            message,
            CreateInteractionResponseFollowup::new()
                .content(translate!(ctx, "ntz-game-set", userA: &actual.name, userB: &rival.name, table: table))
                .allowed_mentions(CreateAllowedMentions::new().all_users(false))
                .components(vec![]),
        )
        .await?;

    Ok(())
}

pub async fn round_lose(
    ctx: Context<'_>,
    inter: &ComponentInteraction,
    nim: &Nim,
    message: MessageId,
) -> Result<(), Error> {
    let current = nim.current_player();

    let table = nim
        .table_cards
        .iter()
        .map(|card| card.name(ctx))
        .collect::<Vec<String>>();
    let table = format!("{:?}", table);

    inter
        .edit_followup(
            ctx,
            message,
            CreateInteractionResponseFollowup::new()
                .content(translate!(ctx, "ntz-round-set", user: &current.name, table: table))
                .allowed_mentions(CreateAllowedMentions::new().all_users(false))
                .components(vec![]),
        )
        .await?;

    Ok(())
}

pub async fn next_turn(
    ctx: Context<'_>,
    inter: &ComponentInteraction,
    nim: &Nim,
    message: MessageId,
) -> Result<(), Error> {
    let actual = nim.current_player();

    let table = nim
        .table_cards
        .iter()
        .map(|card| card.name(ctx))
        .collect::<Vec<String>>();

    let table = format!("{:?}", table);

    let last = nim.table_cards.last().unwrap();
    let rival = nim.players.last().unwrap();

    inter
        .edit_followup(
            ctx,
            message,
            CreateInteractionResponseFollowup::new()
                .content(
                    translate!(ctx, "ntz-round-state", userA: &rival.name, userB: &actual.name, card: last.name(ctx), table: table),
                )
                .allowed_mentions(CreateAllowedMentions::new().all_users(false))
                .components(vec![CreateActionRow::Buttons(vec![CreateButton::new(
                    format!("choose_{}", ctx.id()),
                )
                .style(ButtonStyle::Secondary)
                .label(translate!(ctx, "choose-card-btn"))])]),
        )
        .await?;

    Ok(())
}

pub async fn choose_card(
    ctx: Context<'_>,
    inter: &ComponentInteraction,
    nim: &Nim,
) -> Result<Message, Error> {
    let actual = nim.current_player();
    let mut comps = Vec::new();

    for (index, card) in actual.hand.iter().enumerate() {
        let button = CreateButton::new(format!("{}_card_{}", ctx.id(), index))
            .style(ButtonStyle::Secondary)
            .label(card.name(ctx));

        comps.push(button);
    }

    let msg = inter
        .create_followup(
            ctx,
            CreateInteractionResponseFollowup::new()
                .content(translate!(ctx, "ntz-choose-card"))
                .allowed_mentions(CreateAllowedMentions::new().all_users(false))
                .components(vec![CreateActionRow::Buttons(comps)]),
        )
        .await?;

    Ok(msg)
}

pub async fn next_round(
    ctx: Context<'_>,
    nim: &Nim,
    inter: &ComponentInteraction,
) -> Result<Message, Error> {
    let actual = nim.current_player();

    let message = inter
        .create_followup(
            ctx,
            CreateInteractionResponseFollowup::new()
                .content(translate!(ctx, "ntz-round-first-state", user: &actual.name))
                .allowed_mentions(CreateAllowedMentions::new().all_users(false))
                .components(vec![CreateActionRow::Buttons(vec![CreateButton::new(
                    format!("choose_{}", ctx.id()),
                )
                .style(ButtonStyle::Secondary)
                .label(translate!(ctx, "choose-card-btn"))])]),
        )
        .await?;

    Ok(message)
}

pub async fn first_turn(
    ctx: Context<'_>,
    nim: &Nim,
    inter: &ComponentInteraction,
) -> Result<(), Error> {
    let actual = nim.current_player();

    inter
        .create_response(
            ctx,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .content(translate!(ctx, "ntz-round-first-state", user: &actual.name))
                    .allowed_mentions(CreateAllowedMentions::new().all_users(false))
                    .components(vec![CreateActionRow::Buttons(vec![CreateButton::new(
                        format!("choose_{}", ctx.id()),
                    )
                    .style(ButtonStyle::Secondary)
                    .label(translate!(ctx, "choose-card-btn"))])]),
            ),
        )
        .await?;

    Ok(())
}

pub async fn proposal(ctx: Context<'_>, user: User) -> Result<Message, Error> {
    let message = ctx
        .send(
            CreateReply::default()
                .content(translate!(ctx, "ntz-proposal", user: user.name.to_title_case()))
                .allowed_mentions(CreateAllowedMentions::new().all_users(false))
                .components(vec![CreateActionRow::Buttons(vec![
                    CreateButton::new(format!("accept_{}", ctx.id()))
                        .style(ButtonStyle::Secondary)
                        .label(translate!(ctx, "accept-btn")),
                    CreateButton::new(format!("decline_{}", ctx.id()))
                        .style(ButtonStyle::Secondary)
                        .label(translate!(ctx, "decline-btn")),
                ])]),
        )
        .await?
        .into_message()
        .await?;

    Ok(message)
}
