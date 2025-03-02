use std::time::Duration;

use poise::{
    serenity_prelude::{
        ComponentInteraction, ComponentInteractionCollector, CreateInteractionResponse,
        CreateInteractionResponseFollowup, CreateInteractionResponseMessage, ModalInteraction,
        ModalInteractionCollector, User, UserId,
    },
    Modal,
};

use types::contradiction::{Battle, Contradiction, Player, Role};

use crate::{helpers::get_member, items_auto, Context, Error, Game, Parse};

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
        let collector = ComponentInteractionCollector::new(ctx).timeout(Duration::from_secs(120));

        let modal_collector = ModalInteractionCollector::new(ctx);

        let event = tokio::select! {
            inter = collector.next() => {
                if inter.is_none() {
                    break;
                }

                Event::Interaction(inter.unwrap())
            }
            modal_inter = modal_collector.next() => {
                if modal_inter.is_none() {
                    break;
                }

                Event::Modal(modal_inter.unwrap())
            }
        };

        match event {
            Event::Interaction(inter) => {
                if inter.data.custom_id == format!("{}_accept", ctx.id())
                    && inter.user.id == user.clone().id
                {
                    if crate::can_partial_bet(ctx, inter.user.id, bet).await? {
                        last_inter_player = Some(inter.user.id);
                        last_inter = Some(inter.clone());

                        contradict.init_roles();
                        responses::start(ctx, &inter, &contradict).await?;
                    } else {
                        inter
                            .create_response(
                                ctx,
                                CreateInteractionResponse::UpdateMessage(
                                    CreateInteractionResponseMessage::new()
                                        .content(
                                            "no tienes por lo menos el 80% de dinero para aceptar",
                                        )
                                        .components(vec![]),
                                ),
                            )
                            .await?;

                        return Ok(());
                    }
                }

                if inter.data.custom_id == format!("{}_decline", ctx.id())
                    && inter.user.id == user.clone().id
                {
                    responses::declined(ctx, &inter).await?;
                    return Ok(());
                }

                if contradict
                    .players
                    .iter()
                    .any(|player| player.id == inter.user.id)
                {
                    last_inter_player = Some(inter.user.id);
                    last_inter = Some(inter.clone());

                    if inter.data.custom_id == format!("{}_choose", ctx.id()) {
                        inter.defer_ephemeral(ctx).await?;

                        let msg = responses::choose_object(ctx, &inter, &contradict).await?;

                        let player = contradict.get_mut_player(inter.user.id).unwrap();
                        player.set_ephemeral(msg);
                    }

                    if inter.data.custom_id == format!("{}_bet", ctx.id()) {
                        if contradict.already_bet.contains(&inter.user.id) {
                            inter
                                .create_response(ctx, CreateInteractionResponse::Acknowledge)
                                .await?;
                        } else {
                            let player = contradict.get_player(inter.user.id).unwrap();
                            responses::bet_modal(ctx, &inter, player).await?;
                        }
                    }
                }

                // shield and weapon selection
                if let Some(index) = inter
                    .data
                    .custom_id
                    .strip_prefix(&format!("{}_object_", ctx.id()))
                {
                    if let Ok(index) = index.parse::<usize>() {
                        last_inter_player = Some(inter.user.id);

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
                last_inter_player = Some(inter.user.id);

                if inter.data.custom_id == format!("{}_bet", ctx.id()) {
                    let player = contradict.get_mut_player(inter.user.id);

                    if let Some(player) = player {
                        let modal = BetModal::parse(inter.clone().data)?;
                        let bet = modal.amount.parse::<usize>();

                        if let Ok(bet) = bet {
                            if bet as isize <= player.bios {
                                if bet == 0 && player.bios > 0 {
                                    responses::incorrect_bet(ctx, &inter).await?;
                                } else {
                                    player.bet(bet);
                                    contradict.already_bet.push(inter.user.id);

                                    inter
                                        .create_response(
                                            ctx,
                                            CreateInteractionResponse::Acknowledge,
                                        )
                                        .await?;
                                }
                            } else {
                                responses::incorrect_bet(ctx, &inter).await?;
                            }
                        } else {
                            responses::incorrect_bet(ctx, &inter).await?;
                        }
                    }

                    if contradict.all_bet() {
                        if contradict.is_bet_draw() {
                            let player = contradict.players.first().unwrap();
                            responses::bet_again(ctx, &inter, message.id, player.current_bet)
                                .await?;

                            for player in contradict.players.iter_mut() {
                                player.reset_bet();
                            }

                            contradict.already_bet.clear();
                        } else {
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

                            contradict.already_bet.clear();

                            contradict.selected_shield = None;
                            contradict.selected_weapon = None;

                            if contradict.to_end() {
                                tokio::time::sleep(Duration::from_secs(4)).await;

                                let winner = contradict.get_winner().unwrap();
                                let loser = contradict.get_loser().unwrap();

                                let winner_player = get_member(ctx, winner.id).await?;
                                let winner_read = winner_player.read().await;

                                let winner_message = winner_read
                                    .get_victory_text(
                                        &Game::Contradict.to_string(),
                                        &ctx.data().pool,
                                    )
                                    .await;

                                crate::charge_bet(ctx, winner.id, loser.id, bet).await?;

                                responses::final_result(
                                    ctx,
                                    &inter,
                                    &winner.name,
                                    &loser.name,
                                    winner_message,
                                )
                                .await?;

                                return Ok(());
                            }

                            if contradict.shields.is_empty() {
                                contradict.setup_next_round();
                                contradict.round_info.setup_next();
                                message = responses::new_round(ctx, &inter, &contradict).await?;
                            } else if contradict.only_one_object_left() {
                                contradict.round_info.add_round();

                                contradict.selected_shield = Some(0);
                                contradict.selected_weapon = Some(0);
                                message =
                                    responses::final_bet_phase(ctx, &inter, &contradict).await?;
                            } else {
                                contradict.round_info.add_round();
                                message = responses::new_turn(ctx, &inter, &contradict).await?;
                            }
                        }
                    }
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
