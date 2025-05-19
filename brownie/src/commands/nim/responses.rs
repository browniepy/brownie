use poise::{
    serenity_prelude::{
        ButtonStyle, ComponentInteraction, CreateActionRow, CreateButton,
        CreateInteractionResponse, CreateInteractionResponseFollowup,
        CreateInteractionResponseMessage, Mentionable, Message, MessageId, User,
    },
    CreateReply,
};
use types::{cards::nim_zero::Card, nim_type_zero::Player};

use super::{Context, Error};
use crate::{commands::CommonButton, translate, Parser};

pub struct ErrorRes;

pub struct Response;

struct Button;

impl ErrorRes {
    pub async fn self_accept(ctx: Context<'_>, inter: &ComponentInteraction) -> Result<(), Error> {
        let content = translate!(ctx, "self-accept");

        inter
            .create_response(
                ctx,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content(content)
                        .ephemeral(true),
                ),
            )
            .await?;

        Ok(())
    }

    pub async fn isnt_your_turn(
        ctx: Context<'_>,
        inter: &ComponentInteraction,
    ) -> Result<(), Error> {
        let content = translate!(ctx, "isnt-your-turn");

        inter
            .create_response(
                ctx,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content(content)
                        .ephemeral(true)
                        .components(vec![]),
                ),
            )
            .await?;

        Ok(())
    }

    pub async fn cannot_accept(
        ctx: Context<'_>,
        inter: &ComponentInteraction,
    ) -> Result<(), Error> {
        let content = translate!(ctx, "cannot-accept");

        inter
            .create_response(
                ctx,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new()
                        .content(content)
                        .components(Button::accept_or_decline(ctx, true)),
                ),
            )
            .await?;

        Ok(())
    }

    pub async fn nim_timeout(
        ctx: Context<'_>,
        inter: &ComponentInteraction,
        message_id: MessageId,
        winner: &str,
        loser: &str,
    ) -> Result<(), Error> {
        let content = translate!(ctx, "timeout-vs", winner: winner, loser: loser);

        inter
            .edit_followup(
                ctx,
                message_id,
                CreateInteractionResponseFollowup::new()
                    .content(content)
                    .components(Button::choose(ctx, true))
                    .allowed_mentions(crate::mentions()),
            )
            .await?;

        Ok(())
    }
}

impl Response {
    pub async fn nim_request_bot(ctx: Context<'_>, amount: i64) -> Result<Message, Error> {
        let bet = Parser::abbreviate_number(amount);
        let content = translate!(ctx, "gamble-request-machine", amount: bet);

        let message = ctx
            .send(
                CreateReply::default()
                    .content(content)
                    .components(CommonButton::continue_button(ctx)),
            )
            .await?
            .into_message()
            .await?;

        Ok(message)
    }

    pub async fn nim_request(ctx: Context<'_>, user: &User, amount: i64) -> Result<Message, Error> {
        let bet = Parser::abbreviate_number(amount);
        let content =
            translate!(ctx, "gamble-request", user: user.mention().to_string(), amount: bet);

        let message = ctx
            .send(
                CreateReply::default()
                    .content(content)
                    .components(Button::accept_or_decline(ctx, false))
                    .allowed_mentions(crate::mentions()),
            )
            .await?
            .into_message()
            .await?;

        Ok(message)
    }

    pub async fn nim_declined(
        ctx: Context<'_>,
        inter: &ComponentInteraction,
        user: &User,
    ) -> Result<(), Error> {
        let content = translate!(ctx, "gamble-cancel", user: user.mention().to_string());

        inter
            .create_response(
                ctx,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new()
                        .content(content)
                        .components(Button::accept_or_decline(ctx, true))
                        .allowed_mentions(crate::mentions()),
                ),
            )
            .await?;

        Ok(())
    }

    pub async fn choose_card(
        ctx: Context<'_>,
        inter: &ComponentInteraction,
        player: &Player,
    ) -> Result<MessageId, Error> {
        let content = translate!(ctx, "nim-ur-cards");

        let message = inter
            .create_followup(
                ctx,
                CreateInteractionResponseFollowup::new()
                    .content(content)
                    .components(Button::cards(ctx, player))
                    .allowed_mentions(crate::mentions()),
            )
            .await?;

        Ok(message.id)
    }

    pub async fn nim_start(
        ctx: Context<'_>,
        inter: &ComponentInteraction,
        player: &Player,
    ) -> Result<(), Error> {
        let content = translate!(ctx, "nim-start", user: &player.name);

        inter
            .create_response(
                ctx,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new()
                        .content(content)
                        .components(Button::choose(ctx, false))
                        .allowed_mentions(crate::mentions()),
                ),
            )
            .await?;

        Ok(())
    }

    pub async fn new_round(
        ctx: Context<'_>,
        inter: &ComponentInteraction,
        message_id: MessageId,
        disabled: bool,
        last_card: &Card,
        player: &str,
        total: i32,
        next_player: &str,
    ) -> Result<(), Error> {
        let card = translate!(ctx, &last_card.name());
        let mut content = translate!(ctx, "nim-round-info", user: player, card: card, total: total);

        let subcontent = translate!(ctx, "nim-turn", user: next_player);
        content.push_str(&format!("\n\n{}", subcontent));

        inter
            .edit_followup(
                ctx,
                message_id,
                CreateInteractionResponseFollowup::new()
                    .content(content)
                    .components(Button::choose(ctx, disabled))
                    .allowed_mentions(crate::mentions()),
            )
            .await?;

        Ok(())
    }

    pub async fn nim_end(
        ctx: Context<'_>,
        inter: &ComponentInteraction,
        loser: &str,
        winner: &str,
        card: &Card,
        message_id: MessageId,
        total: i32,
    ) -> Result<(), Error> {
        let card = translate!(ctx, &card.name());
        let content = translate!(ctx, "nim-game-lose", loser: loser, winner: winner, card: &card);

        inter
            .edit_followup(
                ctx,
                message_id,
                CreateInteractionResponseFollowup::new()
                    .content(
                        translate!(ctx, "nim-round-info", user: loser, card: card, total: total),
                    )
                    .components(vec![])
                    .allowed_mentions(crate::mentions()),
            )
            .await?;

        inter
            .create_followup(
                ctx,
                CreateInteractionResponseFollowup::new()
                    .content(content)
                    .allowed_mentions(crate::mentions()),
            )
            .await?;

        Ok(())
    }
}

impl Button {
    fn accept_or_decline(ctx: Context<'_>, all_disabled: bool) -> Vec<CreateActionRow> {
        vec![CreateActionRow::Buttons(vec![
            CreateButton::new(format!("{}_accept", ctx.id()))
                .style(ButtonStyle::Secondary)
                .label(translate!(ctx, "accept"))
                .disabled(all_disabled),
            CreateButton::new(format!("{}_decline", ctx.id()))
                .style(ButtonStyle::Secondary)
                .label(translate!(ctx, "decline"))
                .disabled(all_disabled),
        ])]
    }

    fn choose(ctx: Context<'_>, disabled: bool) -> Vec<CreateActionRow> {
        vec![CreateActionRow::Buttons(vec![CreateButton::new(format!(
            "{}_choose",
            ctx.id()
        ))
        .style(ButtonStyle::Secondary)
        .label(translate!(ctx, "choose-card"))
        .disabled(disabled)])]
    }

    fn cards(ctx: Context<'_>, player: &Player) -> Vec<CreateActionRow> {
        if player.hand.len() <= 5 {
            let mut buttons = Vec::new();

            for (index, card) in player.hand.iter().enumerate() {
                let button = CreateButton::new(format!("{}_card_{}", ctx.id(), index))
                    .style(ButtonStyle::Secondary)
                    .label(card.name())
                    .disabled(card.disabled);

                buttons.push(button);
            }

            vec![CreateActionRow::Buttons(buttons)]
        } else {
            let mut first_row_buttons = Vec::new();
            let mut second_row_buttons = Vec::new();

            for (index, card) in player.hand.iter().enumerate().take(4) {
                let button = CreateButton::new(format!("{}_card_{}", ctx.id(), index))
                    .style(ButtonStyle::Secondary)
                    .label(card.name())
                    .disabled(card.disabled);

                first_row_buttons.push(button);
            }

            for (index, card) in player.hand.iter().enumerate().skip(4) {
                let button = CreateButton::new(format!("{}_card_{}", ctx.id(), index))
                    .style(ButtonStyle::Secondary)
                    .label(translate!(ctx, &card.name()))
                    .disabled(card.disabled);

                second_row_buttons.push(button);
            }

            vec![
                CreateActionRow::Buttons(first_row_buttons),
                CreateActionRow::Buttons(second_row_buttons),
            ]
        }
    }
}
