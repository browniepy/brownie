use std::time::Duration;

use poise::{
    serenity_prelude::{
        ComponentInteraction, ComponentInteractionCollector, ModalInteraction,
        ModalInteractionCollector, User,
    },
    Modal,
};

use types::contradiction::{Battle, Contradiction, Player, Role};

use crate::{Context, Error, Parse};

mod responses;

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
    user: Option<User>,
    bet: Option<String>,
) -> Result<(), Error> {
    let bet = Parse::amount(ctx, ctx.author().id, bet).await?;

    let mut contradict = Contradiction::new(vec![Player::new(ctx.author())]);

    let mut message = if let Some(user) = user {
        contradict.players.push(Player::new(&user));
        responses::accept(ctx, &user, bet).await?
    } else {
        responses::join(ctx, bet).await?
    };

    loop {
        let collector = ComponentInteractionCollector::new(ctx).timeout(Duration::from_secs(120));

        let modal_collector = ModalInteractionCollector::new(ctx).timeout(Duration::from_secs(120));

        let event = tokio::select! {
            inter = collector.next() => Event::Interaction(inter.unwrap()),
            modal_inter = modal_collector.next() => Event::Modal(modal_inter.unwrap()),
        };

        match event {
            Event::Interaction(inter) => {
                if inter.data.custom_id == format!("{}_accept", ctx.id()) {
                    contradict.init_roles();
                    responses::start(ctx, &inter, &contradict).await?;
                }

                if inter.data.custom_id == format!("{}_decline", ctx.id()) {
                    responses::declined(ctx, &inter).await?;
                    break;
                }

                if inter.data.custom_id == format!("{}_join", ctx.id()) {
                    contradict.players.push(Player::new(&inter.user));
                    contradict.init_roles();
                    responses::start(ctx, &inter, &contradict).await?;
                }

                if inter.data.custom_id == format!("{}_choose", ctx.id()) {
                    inter.defer_ephemeral(ctx).await?;

                    let msg = responses::choose_object(ctx, &inter, &contradict).await?;

                    let player = contradict.get_mut_player(inter.user.id).unwrap();
                    player.set_ephemeral(msg);
                }

                if inter.data.custom_id == format!("{}_bet", ctx.id()) {
                    let player = contradict.get_player(inter.user.id).unwrap();
                    responses::bet_modal(ctx, &inter, player).await?;
                }

                // shield and weapon selection
                if let Some(index) = inter
                    .data
                    .custom_id
                    .strip_prefix(&format!("{}_object_", ctx.id()))
                {
                    if let Ok(index) = index.parse::<usize>() {
                        inter.defer(ctx).await?;

                        let player = contradict.get_mut_player(inter.user.id).unwrap();

                        inter
                            .delete_followup(ctx, &player.ephemeral.clone().unwrap().id)
                            .await?;

                        player.delete_ephemeral();

                        match player.role {
                            Role::Defender => {
                                contradict.select_shield(index);
                            }
                            Role::Attacker => {
                                contradict.select_weapon(index);
                            }
                            Role::None => {}
                        }

                        if contradict.all_selected() {
                            responses::bet_phase(ctx, &inter, message.id).await?;
                        }
                    }
                }
            }

            Event::Modal(inter) => {
                if inter.data.custom_id == format!("{}_bet_modal", ctx.id()) {
                    let player = contradict.get_mut_player(inter.user.id);

                    if let Some(player) = player {
                        let modal = BetModal::parse(inter.clone().data)?;
                        let bet = modal.amount.parse::<usize>()?;

                        player.bet(bet);
                        contradict.already_bet.push(inter.user.id);
                    }

                    if contradict.all_bet() {
                        if contradict.to_end() {
                            let _loser = contradict.get_loser().unwrap();
                            let _winner = contradict.get_winner().unwrap();

                            // end game and show winner and loser
                        } else {
                            // start comparison with update message

                            let reaction = contradict.battle();

                            responses::comparison(
                                ctx,
                                &inter,
                                message.id,
                                &mut contradict,
                                reaction,
                            )
                            .await?;

                            // reset player states
                            contradict.delete_stock();

                            for player in contradict.players.iter_mut() {
                                player.confirm_bet();
                                player.reset_bet();
                            }
                        }

                        tokio::time::sleep(Duration::from_secs(5)).await;

                        if contradict.shields.is_empty() {
                            contradict.setup_next_round();
                            message = responses::new_round(ctx, &inter, &contradict).await?;
                        } else if contradict.only_one_object_left() {
                            message = responses::final_bet_phase(ctx, &inter, &contradict).await?;
                        } else {
                            message = responses::new_turn(ctx, &inter).await?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
