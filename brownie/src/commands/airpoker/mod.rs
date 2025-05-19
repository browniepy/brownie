use crate::{Context, Error, Parser};
use poise::serenity_prelude::{
    ComponentInteraction, ComponentInteractionCollector, ModalInteraction,
    ModalInteractionCollector, User,
};
use tokio::select;

use std::time::Instant;

mod responses;
use responses::Response;

use types::airpoker::{AirPoker, Player};

#[derive(poise::Modal)]
struct BetModal {
    amount: String,
}

#[derive(Debug)]
enum Signal {
    SelectCardRound { inter: ComponentInteraction },
    BetRound { inter: ComponentInteraction },
    GameEnd { inter: ComponentInteraction },
    BetRoundEnd,
    SelectCardEnd,
    Tick,
}

enum Event {
    Interaction(ComponentInteraction),
    ModalInter(ModalInteraction),
    Timeout,
}

// choose, breathe
// bet, breathe

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "gambling"
)]
pub async fn airpoker(ctx: Context<'_>, user: User, bios: Option<String>) -> Result<(), Error> {
    if user.id == ctx.author().id {
        return Err("hola".into());
    }

    let bet = Parser::amount(ctx, ctx.author().id, bios, 500).await?;

    let mut airpoker = AirPoker::new(Player::new(ctx.author().clone()), Player::new(user.clone()));

    let mut elapsed = Instant::now();

    loop {
        let collector = ComponentInteractionCollector::new(ctx);
        let modal_collector = ModalInteractionCollector::new(ctx);

        let timeout = async {
            if elapsed.elapsed().as_secs() > 60 {
                std::future::ready(()).await
            } else {
                std::future::pending().await
            }
        };

        let event = select! {
            inter = collector.next() => Event::Interaction(inter.unwrap()),
            modal_inter = modal_collector.next() => Event::ModalInter(modal_inter.unwrap()),
            _ = timeout => Event::Timeout,
        };

        match event {
            Event::Interaction(inter) => {
                if inter.user.id != ctx.author().id && inter.user.id != user.id {
                    continue;
                }

                elapsed = Instant::now();

                if inter.data.custom_id == format!("{}_accept", ctx.id()) {
                    airpoker.deal_cards();
                }

                if inter.data.custom_id == format!("{}_decline", ctx.id()) {
                    return Ok(());
                }

                if inter.data.custom_id == format!("{}_choose", ctx.id()) {
                    inter.defer_ephemeral(ctx).await?;

                    let player = airpoker.get_player(inter.user.id).unwrap();
                }

                if inter.data.custom_id == format!("{}_bet", ctx.id()) {
                    let player = airpoker.get_player(inter.user.id).unwrap();

                    if player.get_betable_air_bios() < airpoker.blind as usize {
                        Response::inform_all_in(ctx, inter).await?;
                    } else {
                        Response::open_bet_modal(ctx, inter, player).await?;
                    }
                }
            }
            Event::ModalInter(inter) => {}
            Event::Timeout => {
                // Handle timeout logic here
                break;
            }
        }
    }

    Ok(())
}
