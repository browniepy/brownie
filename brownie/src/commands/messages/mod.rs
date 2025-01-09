use poise::serenity_prelude::{
    ComponentInteraction, CreateInteractionResponse, CreateInteractionResponseMessage,
};

use crate::{translation::translate, Context, Error};

pub mod dth;

pub mod work;

pub mod nim;

pub mod oldmaid;

pub mod airpoker;

pub mod bj;

pub async fn wrong_interaction(
    ctx: Context<'_>,
    inter: &ComponentInteraction,
) -> Result<(), Error> {
    inter
        .create_response(
            ctx,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(translate!(ctx, "wrong-inter"))
                    .ephemeral(true),
            ),
        )
        .await?;

    Ok(())
}
