use std::time::Duration;

use poise::{
    serenity_prelude::{CreateInteractionResponse, Message, MessageId},
    Modal,
};
use types::contradiction::Battle;

use super::{responses, BetModal, CollectorRes, Context, Contradiction, Error, ModalInteraction};
use crate::{helpers::*, Game};

pub async fn handle_modal(
    ctx: Context<'_>,
    inter: ModalInteraction,
    contradict: &mut Contradiction,
    bet: i32,
    message_id: MessageId,
    message: &mut Message,
) -> Result<CollectorRes, Error> {
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
                        .create_response(ctx, CreateInteractionResponse::Acknowledge)
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
            responses::bet_again(ctx, &inter, message_id, player.current_bet).await?;

            for player in contradict.players.iter_mut() {
                player.reset_bet();
            }

            contradict.already_bet.clear();
        } else {
            let reaction = contradict.battle();

            responses::comparison(ctx, &inter, message_id, contradict, reaction).await?;

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
                    .get_victory_text(&Game::Contradict.to_string(), &ctx.data().pool)
                    .await;

                crate::charge_bet(ctx, winner.id, loser.id, bet).await?;

                responses::final_result(ctx, &inter, &winner.name, &loser.name, winner_message)
                    .await?;

                return Ok(CollectorRes::Finish);
            }

            if contradict.shields.is_empty() {
                contradict.setup_next_round();
                contradict.round_info.setup_next();
                *message = responses::new_round(ctx, &inter, contradict).await?;
            } else if contradict.only_one_object_left() {
                contradict.round_info.add_round();

                contradict.selected_shield = Some(0);
                contradict.selected_weapon = Some(0);
                *message = responses::final_bet_phase(ctx, &inter, contradict).await?;
            } else {
                contradict.round_info.add_round();
                *message = responses::new_turn(ctx, &inter, contradict).await?;
            }
        }
    }

    Ok(CollectorRes::Ok)
}
