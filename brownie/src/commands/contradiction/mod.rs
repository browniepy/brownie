mod responses;
use responses::{ModalRes, Response};

use std::time::Duration;

use poise::{
    serenity_prelude::{
        ComponentInteraction, ComponentInteractionCollector, ModalInteraction,
        ModalInteractionCollector, User, UserId,
    },
    Modal,
};
use types::contradiction::{Battle, Contradiction, Player, Role};

use super::CommonRes;
use crate::{Context, Error, Parse};

enum Event {
    Inter(ComponentInteraction),
    ModalInter(ModalInteraction),
}

#[derive(poise::Modal)]
struct BetModal {
    bios: Option<String>,
}

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "gambling"
)]
pub async fn contradict(ctx: Context<'_>, user: User, bios: Option<String>) -> Result<(), Error> {
    if user.id == ctx.author().id {
        return Err("cannot play with yourself".into());
    }

    let bet = Parse::amount(ctx, ctx.author().id, bios).await?;

    let mut contradict = Contradiction::new(vec![Player::new(ctx.author())]);

    let mut message = Response::request(ctx, &user, bet).await?;

    let mut last_inter_player: Option<UserId> = None;
    let mut last_inter: Option<ComponentInteraction> = None;

    loop {
        let author_id = ctx.author().id;
        let inter_collector = ComponentInteractionCollector::new(ctx)
            .timeout(Duration::from_secs(80))
            .filter(move |i| i.user.id == user.id || i.user.id == author_id);

        let modal_collector = ModalInteractionCollector::new(ctx);

        let event = tokio::select! {
            inter = inter_collector.next() => {
                if inter.is_none() {
                    break;
                }

                Event::Inter(inter.unwrap())
            }
            modal_inter = modal_collector.next() => {
                if modal_inter.is_none() {
                    break;
                }

                Event::ModalInter(modal_inter.unwrap())
            }
        };

        match event {
            Event::Inter(inter) => {
                last_inter = Some(inter.clone());

                if inter.data.custom_id == format!("{}_accept", ctx.id()) {
                    if inter.user.id == user.id {
                        if !crate::can_partial_bet(ctx, inter.user.id, bet).await? {
                            CommonRes::cannot_accept(ctx, &inter).await?;
                            return Ok(());
                        }

                        last_inter_player = Some(inter.user.id);

                        contradict.players.push(Player::new(&inter.user));
                        contradict.init_roles();

                        Response::start(ctx, &inter, &contradict).await?;
                    } else {
                        CommonRes::self_accept(ctx, &inter).await?;
                    }
                }

                if inter.data.custom_id == format!("{}_decline", ctx.id()) {
                    Response::declined(ctx, &inter, &inter.user).await?;
                    return Ok(());
                }

                if inter.data.custom_id == format!("{}_choose", ctx.id()) {
                    if contradict.already_selected.contains(&inter.user.id) {
                        CommonRes::already_action(ctx, &inter).await?;
                    } else {
                        inter.defer_ephemeral(ctx).await?;
                        let message = Response::choose(ctx, &inter, &contradict).await?;

                        if let Some(player) = contradict.get_mut_player(inter.user.id) {
                            player.set_ephemeral(message);
                        }
                    }
                }

                if inter.data.custom_id == format!("{}_bet", ctx.id()) {
                    if contradict.already_bet.contains(&inter.user.id) {
                        CommonRes::already_action(ctx, &inter).await?;
                    } else if let Some(player) = contradict.get_player(inter.user.id) {
                        ModalRes::bet(ctx, &inter, player).await?;
                    }
                }

                if let Some(index) = inter
                    .data
                    .custom_id
                    .strip_prefix(&format!("{}_object_", ctx.id()))
                {
                    if let Ok(index) = index.parse::<usize>() {
                        inter.defer(ctx).await?;

                        let player = contradict.get_mut_player(inter.user.id).unwrap();

                        if let Some(ephemeral) = &player.ephemeral {
                            inter.delete_followup(ctx, ephemeral.id).await?;
                            player.delete_ephemeral();
                        }

                        match player.role {
                            Role::Defender => contradict.select_shield(index),
                            _ => contradict.select_weapon(index),
                        }

                        contradict.already_selected.push(inter.user.id);

                        if contradict.all_selected() {
                            Response::bet_phase(ctx, &inter, message.id).await?;
                        }
                    }
                }
            }

            Event::ModalInter(inter) => {
                let player = contradict.get_mut_player(inter.user.id).unwrap();

                let modal = BetModal::parse(inter.clone().data).unwrap();
                let bios_bet = modal.bios.unwrap_or(if player.bios > 0 {
                    1.to_string()
                } else {
                    0.to_string()
                });

                let bios_bet = Parse::abbreviation_to_number(&bios_bet);

                if let Ok(bet) = bios_bet {
                    if bet as isize <= player.bios {
                        if bet == 0 && player.bios > 0 {
                            CommonRes::incorrect_bet(ctx, &inter).await?;
                        } else {
                            player.bet(bet as usize);
                            contradict.already_bet.push(inter.user.id);

                            CommonRes::modal_your_bet_res(ctx, &inter, bet).await?;
                        }
                    } else {
                        CommonRes::incorrect_bet(ctx, &inter).await?;
                    }
                } else {
                    CommonRes::incorrect_bet(ctx, &inter).await?;
                }

                if contradict.all_bet() {
                    if contradict.is_bet_draw() {
                        Response::bet_draw(ctx, &inter, message.id).await?;

                        for player in contradict.players.iter_mut() {
                            player.reset_bet();
                        }

                        contradict.already_bet.clear();
                    } else {
                        let reaction = contradict.battle();

                        Response::comparison(ctx, &inter, message.id, &mut contradict, reaction)
                            .await?;

                        contradict.delete_stock();
                        contradict.already_bet.clear();
                        contradict.already_selected.clear();
                        contradict.selected_shield = None;
                        contradict.selected_weapon = None;

                        for player in contradict.players.iter_mut() {
                            player.confirm_bet();
                            player.reset_bet();
                        }

                        if contradict.to_end() {
                            tokio::time::sleep(Duration::from_secs(3)).await;

                            let winner = contradict.get_winner().unwrap();
                            let loser = contradict.get_loser().unwrap();

                            crate::charge_bet(ctx, winner.id, loser.id, bet).await?;

                            Response::final_result(ctx, &inter, &winner.name, &loser.name).await?;

                            return Ok(());
                        }

                        tokio::time::sleep(Duration::from_secs(3)).await;

                        if contradict.empty_objects() {
                            contradict.setup_next_round();
                            contradict.round_info.setup_next();
                            message = Response::new_round(ctx, &inter, &contradict).await?;
                        } else {
                            contradict.round_info.add_round();
                            message = Response::new_round(ctx, &inter, &contradict).await?;
                        }
                    }
                }
            }
        }
    }

    if let Some(id) = last_inter_player {
        let winner = contradict.get_player(id).unwrap();
        let loser = contradict.players.iter().find(|p| p.id != id).unwrap();

        crate::charge_bet(ctx, winner.id, loser.id, bet).await?;

        if let Some(inter) = last_inter {
            CommonRes::vs_timeout(ctx, &inter, message.id, &winner.name, &loser.name).await?;
        }
    }

    Ok(())
}
