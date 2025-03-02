use std::time::Duration;

use poise::serenity_prelude::{
    ComponentInteraction, ComponentInteractionCollector, CreateInteractionResponseFollowup,
    ModalInteraction, ModalInteractionCollector, User,
};

use types::contradiction::{Contradiction, Player, Role};

use crate::{items_auto, Context, Error, Parse};

mod responses;

enum CollectorRes {
    Finish,
    Ok,
}

mod collector;
use collector::handle_inter;

mod modal_collector;
use modal_collector::handle_modal;

enum Event {
    Interaction(ComponentInteraction),
    Modal(ModalInteraction),
}

#[derive(poise::Modal)]
struct BetModal {
    amount: String,
}

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "gambling"
)]
pub async fn contradiction(
    ctx: Context<'_>,
    user: User,
    money: Option<String>,
    #[autocomplete = "items_auto"] item: Option<String>,
    amount: Option<i32>,
) -> Result<(), Error> {
    let bet = Parse::amount(ctx, ctx.author().id, money).await?;

    let mut contradict = Contradiction::new(vec![Player::new(ctx.author())]);

    let mut message = {
        contradict.players.push(Player::new(&user));
        responses::accept(ctx, &user, bet).await?
    };

    let mut last_inter_player = None;
    let mut last_inter = None;

    loop {
        let author_id = ctx.author().id;
        let collector = ComponentInteractionCollector::new(ctx)
            .filter(move |i| i.user.id == author_id || i.user.id == user.id)
            .timeout(Duration::from_secs(120));

        let modal_collector = ModalInteractionCollector::new(ctx);

        let event = tokio::select! {
            inter = collector.next() => {
                if inter.is_none() {
                    break;
                }

                Event::Interaction(inter.unwrap())
            }
            modal_inter = modal_collector.next() => {
                Event::Modal(modal_inter.unwrap())
            }
        };

        match event {
            Event::Interaction(inter) => {
                last_inter_player = Some(inter.user.id);
                last_inter = Some(inter.clone());
                handle_inter(ctx, inter, &mut contradict, bet, &user, message.id).await?;
            }

            Event::Modal(inter) => {
                if let CollectorRes::Finish =
                    handle_modal(ctx, inter, &mut contradict, bet, message.id, &mut message).await?
                {
                    return Ok(());
                }
            }
        }
    }

    tracing::warn!("command timeout");

    if let Some(last_inter_player) = last_inter_player {
        let winner = contradict.get_player(last_inter_player).unwrap();

        let loser = contradict
            .players
            .iter()
            .find(|player| player.id != last_inter_player)
            .unwrap();

        crate::charge_bet(ctx, winner.id, loser.id, bet).await?;

        let content = format!("El jugador {} ha ganado por tiempo", winner.name);

        if let Some(inter) = last_inter {
            inter
                .edit_followup(
                    ctx,
                    message.id,
                    CreateInteractionResponseFollowup::new()
                        .content(content)
                        .components(vec![]),
                )
                .await?;
            return Ok(());
        }
    }

    Err("command timeout error".into())
}
