mod responses;
use responses::{ErrorRes, Response};

use std::time::Duration;

use crate::{Context, Error, Parse};
use poise::serenity_prelude::{ComponentInteraction, ComponentInteractionCollector, Message, User};
use types::nim_type_zero::{Nim, Player};

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "gambling"
)]
pub async fn nim(ctx: Context<'_>, user: User, bios: Option<String>) -> Result<(), Error> {
    if user.id == ctx.author().id {
        return Err("cannot play with yourself".into());
    }

    let bet = Parse::amount(ctx, ctx.author().id, bios).await?;

    let mut nim = Nim::new(Player::new(ctx.author()), bet);
    let mut message = Response::nim_request(ctx, &user, bet).await?;

    let mut last_inter_player = None;
    let mut last_inter = None;

    let author_id = ctx.author().id;
    while let Some(inter) = ComponentInteractionCollector::new(ctx)
        .filter(move |mci| mci.user.id == user.id || mci.user.id == author_id)
        .timeout(Duration::from_secs(40))
        .await
    {
        last_inter = Some(inter.clone());

        if inter.data.custom_id == format!("{}_accept", ctx.id()) {
            if inter.user.id == user.id {
                if !crate::can_partial_bet(ctx, user.id, bet).await? {
                    ErrorRes::cannot_accept(ctx, &inter).await?;
                    return Ok(());
                }
                last_inter_player = Some(inter.user.id);

                nim.add_player(Player::new(&inter.user))?;
                nim.deal_cards();
                Response::nim_start(ctx, &inter, nim.current_player()).await?;
            } else {
                ErrorRes::self_accept(ctx, &inter).await?;
            }
        }

        if inter.data.custom_id == format!("{}_decline", ctx.id()) {
            Response::nim_declined(ctx, &inter, &inter.user).await?;
            return Ok(());
        }

        if inter.data.custom_id == format!("{}_choose", ctx.id()) {
            if inter.user.id == nim.current_player().id {
                inter.defer_ephemeral(ctx).await?;
                let message_id = Response::choose_card(ctx, &inter, nim.current_player()).await?;
                nim.ephemeral = Some(message_id);
            } else {
                ErrorRes::isnt_your_turn(ctx, &inter).await?;
            }
        }

        if let Some(index) = &inter
            .data
            .custom_id
            .strip_prefix(&format!("{}_card_", ctx.id()))
        {
            if let Ok(index) = index.parse::<usize>() {
                inter.defer(ctx).await?;
                last_inter_player = Some(inter.user.id);

                if let Some(id) = nim.ephemeral {
                    inter.delete_followup(ctx, id).await?;
                    nim.ephemeral = None;
                }

                nim.play_card(index);

                let res = process_after_card_played(ctx, &inter, &mut message, &mut nim).await?;
                if res.has_to_finish() {
                    let winner = nim.get_winner().unwrap();
                    let loser = nim.get_loser().unwrap();

                    crate::charge_bet(ctx, winner.id, loser.id, bet).await?;

                    Response::nim_end(ctx, &inter, &loser.name, &winner.name).await?;
                    return Ok(());
                }
            }
        }
    }

    if let Some(id) = last_inter_player {
        let winner = nim.get_player(id);
        let loser = nim.players.iter().find(|p| p.id != id).unwrap();

        crate::charge_bet(ctx, winner.id, loser.id, bet).await?;

        if let Some(inter) = last_inter {
            ErrorRes::nim_timeout(ctx, &inter, message.id, &winner.name, &loser.name).await?;
        }
    }

    Ok(())
}

async fn process_after_card_played(
    ctx: Context<'_>,
    inter: &ComponentInteraction,
    message: &mut Message,
    nim: &mut Nim,
) -> Result<CardPlayedRes, Error> {
    if nim.table_value() > 9 {
        nim.mut_rival_player().wins += 1;

        nim.table_cards.clear();
        nim.deal_cards();

        let loser = nim.current_player();
        let winner = nim.rival_player();

        Response::round_lose(ctx, inter, message.id, &loser.name, &winner.name).await?;
        tokio::time::sleep(Duration::from_secs(3)).await;

        if nim.has_winner() {
            return Ok(CardPlayedRes::FinishGame);
        }

        nim.next_player();
        let actual = nim.current_player();
        *message = Response::new_game(ctx, inter, &actual.name).await?;
    } else {
        nim.next_player();
        nim.check_hand();

        let actual = nim.current_player();
        Response::new_round(
            ctx,
            inter,
            message.id,
            false,
            nim.last_played_card(),
            &actual.name,
        )
        .await?;
    }

    Ok(CardPlayedRes::ContinuePlaying)
}

enum CardPlayedRes {
    ContinuePlaying,
    FinishGame,
}

impl CardPlayedRes {
    fn has_to_finish(&self) -> bool {
        matches!(self, CardPlayedRes::FinishGame)
    }
}
