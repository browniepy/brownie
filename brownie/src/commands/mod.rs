use crate::{translate, Context, Error};
use poise::serenity_prelude::{
    ButtonStyle, ComponentInteraction, CreateActionRow, CreateButton, CreateInteractionResponse,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage, MessageId,
    ModalInteraction,
};

pub mod airpoker;
pub mod autocomplete;
pub mod blackjack;
pub mod check;
pub mod choice;
pub mod club;
pub mod contradiction;
pub mod ecard;
pub mod give;
pub mod greeting;
pub mod nim;
pub mod profile;
pub mod rewards;
pub mod roulette;
pub mod rr;
pub mod store;
pub mod work;

pub struct CommonRes;

pub struct CommonButton;

impl CommonRes {
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
                        .components(CommonButton::accept_or_decline(ctx, true)),
                ),
            )
            .await?;

        Ok(())
    }

    pub async fn already_action(
        ctx: Context<'_>,
        inter: &ComponentInteraction,
    ) -> Result<(), Error> {
        let content = translate!(ctx, "already-action");

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

    pub async fn incorrect_bet(ctx: Context<'_>, inter: &ModalInteraction) -> Result<(), Error> {
        let content = translate!(ctx, "incorrect-bet");

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

    pub async fn know_modal(ctx: Context<'_>, inter: &ModalInteraction) -> Result<(), Error> {
        inter
            .create_response(ctx, CreateInteractionResponse::Acknowledge)
            .await?;

        Ok(())
    }

    pub async fn modal_your_bet_res(
        ctx: Context<'_>,
        inter: &ModalInteraction,
        bet: i32,
    ) -> Result<(), Error> {
        let content = translate!(ctx, "your-bet-res", amount: bet);

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

    pub async fn vs_timeout(
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
                    .components(CommonButton::timeout(ctx))
                    .allowed_mentions(crate::mentions()),
            )
            .await?;

        Ok(())
    }
}

impl CommonButton {
    pub fn pagination(ctx: Context<'_>, prev_dis: bool, next_dis: bool) -> Vec<CreateActionRow> {
        vec![CreateActionRow::Buttons(vec![
            CreateButton::new(format!("{}_prev", ctx.id()))
                .style(ButtonStyle::Secondary)
                .label(translate!(ctx, "prev"))
                .disabled(prev_dis),
            CreateButton::new(format!("{}_next", ctx.id()))
                .style(ButtonStyle::Secondary)
                .label(translate!(ctx, "next"))
                .disabled(next_dis),
        ])]
    }

    pub fn continue_button(ctx: Context<'_>) -> Vec<CreateActionRow> {
        vec![CreateActionRow::Buttons(vec![CreateButton::new(format!(
            "{}_continue",
            ctx.id()
        ))
        .style(ButtonStyle::Secondary)
        .label(translate!(ctx, "continue"))])]
    }

    pub fn timeout(ctx: Context<'_>) -> Vec<CreateActionRow> {
        vec![CreateActionRow::Buttons(vec![CreateButton::new(format!(
            "{}_timeout",
            ctx.id()
        ))
        .style(ButtonStyle::Secondary)
        .label("Timeout")
        .disabled(true)])]
    }

    pub fn accept_or_decline(ctx: Context<'_>, all_disabled: bool) -> Vec<CreateActionRow> {
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
}
