use super::{
    check::{self_can_gamble, user_can_gamble},
    choice::Game,
};
use crate::{
    serenity::{ComponentInteraction, ComponentInteractionCollector, Message, User, UserId},
    Context, Duration, Error, Parser,
};
use responses::{ErrorRes, Response};
use types::nim_type_zero::{Nim, Player};

mod responses;

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    check = "self_can_gamble",
    category = "gambling"
)]
pub async fn nim(ctx: Context<'_>, user: Option<User>, bios: Option<String>) -> Result<(), Error> {
    if let Some(user) = &user {
        if user.id == ctx.author().id {
            return Err("cannot play with yourself".into());
        }

        user_can_gamble(ctx, user.clone()).await?;
    }

    let bet = Parser::amount(ctx, ctx.author().id, bios, 500).await?;

    let mut nim = Nim::new(Player::new(Some(ctx.author()), false), bet);

    let mut message = match user {
        Some(ref user) => Response::nim_request(ctx, user, bet).await?,
        None => Response::nim_request_bot(ctx, bet).await?,
    };

    crate::set_gamble(ctx, ctx.author().id).await?;

    let mut last_interaction = None;

    let user_id = if let Some(ref user) = user {
        user.id
    } else {
        UserId::default()
    };

    let author_id = ctx.author().id;

    while let Some(interaction) = ComponentInteractionCollector::new(ctx)
        .timeout(Duration::from_secs(20))
        .await
    {
        if interaction.user.id != user_id && interaction.user.id != author_id {
            continue;
        }

        last_interaction = Some(interaction.clone());

        if interaction.data.custom_id == format!("{}_continue", ctx.id()) {
            nim.add_player(Player::new(
                Some(&ctx.http().get_user(UserId::new(896535593641734164)).await?),
                true,
            ))?;
            nim.deal_cards();
            Response::nim_start(ctx, &interaction, nim.current_player()).await?;
        }

        if interaction.data.custom_id == format!("{}_accept", ctx.id()) {
            if interaction.user.id == user.clone().unwrap().id {
                if super::check::user_can_gamble(ctx, interaction.user.clone())
                    .await
                    .is_err()
                {
                    ErrorRes::cannot_accept(ctx, &interaction).await?;
                    return Ok(());
                }

                if !crate::can_partial_bet(ctx, user.clone().unwrap().id, bet).await? {
                    ErrorRes::cannot_accept(ctx, &interaction).await?;
                    return Ok(());
                }

                crate::set_gamble(ctx, user.clone().unwrap().id).await?;

                nim.add_player(Player::new(Some(&interaction.user), false))?;
                nim.deal_cards();
                Response::nim_start(ctx, &interaction, nim.current_player()).await?;
            } else {
                ErrorRes::self_accept(ctx, &interaction).await?;
            }
        }

        if interaction.data.custom_id == format!("{}_decline", ctx.id()) {
            crate::free_gamble(ctx, vec![ctx.author().id]).await?;
            Response::nim_declined(ctx, &interaction, &interaction.user).await?;
            return Ok(());
        }

        if interaction.data.custom_id == format!("{}_choose", ctx.id()) {
            if interaction.user.id == nim.current_player().id {
                interaction.defer_ephemeral(ctx).await?;
                let message_id =
                    Response::choose_card(ctx, &interaction, nim.current_player()).await?;
                nim.ephemeral = Some(message_id);
            } else {
                ErrorRes::isnt_your_turn(ctx, &interaction).await?;
            }
        }

        if let Some(index) = &interaction
            .data
            .custom_id
            .strip_prefix(&format!("{}_card_", ctx.id()))
        {
            if let Ok(index) = index.parse::<usize>() {
                interaction.defer(ctx).await?;

                if let Some(id) = nim.ephemeral {
                    interaction.delete_followup(ctx, id).await?;
                    nim.ephemeral = None;
                }

                nim.play_card(index).await?;

                let result =
                    played_card_process(ctx, &interaction, &mut message, &mut nim, bet).await?;

                if result.has_to_finish() {
                    return Ok(());
                }
            }
        }
    }

    crate::free_gamble(
        ctx,
        nim.players
            .iter()
            .map(|player| player.id)
            .collect::<Vec<UserId>>(),
    )
    .await?;

    if let Some(interaction) = last_interaction {
        let user_id = interaction.user.id;

        let winner = nim.get_player(user_id);
        let loser = nim.players.iter().find(|p| p.id != user_id).unwrap();

        if !nim.players.iter().any(|player| player.is_bot()) {
            crate::charge_bet(ctx, winner.id, loser.id, bet, Game::NimTypeZero).await?;
        }

        if !winner.is_bot() {
            crate::charge_single_bet(ctx, winner.id, bet, true).await?;
        }

        if !loser.is_bot() {
            crate::charge_single_bet(ctx, loser.id, bet, false).await?;
        }

        ErrorRes::nim_timeout(ctx, &interaction, message.id, &winner.name, &loser.name).await?;
    }

    Ok(())
}

async fn played_card_process(
    ctx: Context<'_>,
    interaction: &ComponentInteraction,
    message: &mut Message,
    nim: &mut Nim,
    bet: i64,
) -> Result<CardPlayedRes, Error> {
    if nim.table_value() > 9 {
        let loser = nim.current_player();
        let winner = nim.rival_player();

        if !nim.players.iter().any(|player| player.is_bot()) {
            crate::charge_bet(ctx, winner.id, loser.id, bet, Game::NimTypeZero).await?;
        }

        if !winner.is_bot() {
            crate::charge_single_bet(ctx, winner.id, bet, true).await?;
        }

        if !loser.is_bot() {
            crate::charge_single_bet(ctx, loser.id, bet, false).await?;
        }

        crate::free_gamble(
            ctx,
            nim.players
                .iter()
                .map(|player| player.id)
                .collect::<Vec<UserId>>(),
        )
        .await?;

        Response::nim_end(
            ctx,
            interaction,
            &loser.name,
            &winner.name,
            nim.last_played_card(),
            message.id,
            nim.table_value(),
        )
        .await?;
        return Ok(CardPlayedRes::FinishGame);
    } else {
        nim.next_player();
        nim.check_hand();

        let actual = nim.current_player();
        let other = nim.rival_player();

        Response::new_round(
            ctx,
            interaction,
            message.id,
            nim.current_player().is_bot(),
            nim.last_played_card(),
            &other.name,
            nim.table_value(),
            &actual.name,
        )
        .await?;

        if actual.is_bot() {
            nim.bot_play().await?;

            Box::pin(played_card_process(ctx, interaction, message, nim, bet)).await?;

            if nim.table_value() > 9 {
                return Ok(CardPlayedRes::FinishGame);
            }
        }
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
