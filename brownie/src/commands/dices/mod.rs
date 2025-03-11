use std::time::Duration;

use poise::{
    serenity_prelude::{
        ComponentInteraction, ComponentInteractionCollector, EditMessage, ModalInteraction,
        ModalInteractionCollector,
    },
    CreateReply, Modal,
};
use types::dices::{Dices, Player, Selection};

use crate::{charge_single_bet, dices_auto, get_member, Context, Error, Parse};

mod responses;

#[derive(poise::Modal)]
struct BetModal {
    bet: Option<String>,
    choice: String,
}

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "gambling"
)]
pub async fn dices(
    ctx: Context<'_>,
    #[autocomplete = "dices_auto"] choice: String,
    choice_bet: Option<String>,
) -> Result<(), Error> {
    let bet = Parse::amount(ctx, ctx.author().id, choice_bet).await?;

    let mut dices = Dices::new(Player::new(ctx.author(), bet));
    dices.roll_dices();

    let choice_parse = Parse::dice_choice(choice.clone());
    let number_parse = Parse::dice_choice_num(choice.clone());

    match (choice_parse, number_parse) {
        (Ok(choice_value), Err(_)) => {
            dices.player.choice(choice_value);
            println!("choice");
        }
        (Err(_), Ok(number_value)) => {
            dices.player.number(number_value);
            println!("number");
        }
        (Ok(_), Ok(_)) => {
            return Err("cannot select both".into());
        }
        (Err(string_err), Err(number_err)) => {
            if Parse::choice_kind(choice).is_num() {
                return Err(number_err);
            }

            return Err(string_err);
        }
    }

    let message = responses::initial(ctx, &dices.player).await?;

    tokio::time::sleep(Duration::from_secs(5)).await;

    responses::result(ctx, message, &dices).await?;

    Ok(())
}
