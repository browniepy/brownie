use crate::{translate, Context, Error};
use poise::{
    serenity_prelude::{
        ButtonStyle, ComponentInteraction, CreateActionRow, CreateButton, CreateInputText,
        CreateInteractionResponse, CreateInteractionResponseFollowup,
        CreateInteractionResponseMessage, CreateModal, InputTextStyle, Message, MessageId,
    },
    CreateReply,
};
use types::airpoker::Player;

pub struct Response;

impl Response {
    pub async fn inform_all_in(ctx: Context<'_>, inter: ComponentInteraction) -> Result<(), Error> {
        let content = translate!(ctx, "inform-all-in");
        inter
            .create_response(
                ctx,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .ephemeral(true)
                        .content(content),
                ),
            )
            .await?;

        Ok(())
    }

    pub async fn your_cards(
        ctx: Context<'_>,
        inter: ComponentInteraction,
        player: &Player,
    ) -> Result<(), Error> {
        let cards = player
            .hand
            .iter()
            .map(|card| card.value().to_string())
            .collect::<Vec<_>>();

        let content = translate!(ctx, "your-cards", cards: cards.join(", "));

        inter
            .create_response(
                ctx,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .ephemeral(true)
                        .content(content),
                ),
            )
            .await?;

        Ok(())
    }

    pub async fn open_bet_modal(
        ctx: Context<'_>,
        inter: ComponentInteraction,
        player: &Player,
    ) -> Result<(), Error> {
        let title = translate!(ctx, "air-modal-title");
        let question = translate!(ctx, "air-modal-question");
        let placeholder = translate!(ctx, "air-modal-placeholder",
            bios: player.get_betable_air_bios());

        inter
            .create_response(
                ctx,
                CreateInteractionResponse::Modal(CreateModal::new("hola", title).components(
                    vec![CreateActionRow::InputText(
                CreateInputText::new(InputTextStyle::Short, question, "hola")
                .placeholder(placeholder)
            )],
                )),
            )
            .await?;

        Ok(())
    }
}
