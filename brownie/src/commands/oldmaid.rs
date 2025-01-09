#![allow(dead_code)]

use super::messages::oldmaid as message;
use crate::{get_member, Context, Error};
use inflector::Inflector;
use poise::serenity_prelude::{ComponentInteraction, ComponentInteractionCollector, User, UserId};
use tokio::{select, sync::mpsc};
use types::{
    cards::poker::Card,
    oldmaid::{Oldmaid, Player},
};

enum Event {
    Interaction(ComponentInteraction),
    Receiver(Signal),
    Timeout,
}

enum Signal {
    GameEnd(ComponentInteraction, Card),
    Timeout(ComponentInteraction),
    StartTurn(ComponentInteraction, Card),
}

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "gambling"
)]
pub async fn oldmaid(
    ctx: Context<'_>,
    user: Option<User>,
    amount: Option<i32>,
) -> Result<(), Error> {
    if let Some(amount) = amount {
        let author = get_member(ctx, ctx.author().id).await?;
        let read = author.read().await;

        if read.balance < amount {
            return Err("You don't have enough money".into());
        }
    }

    let mut _msg = message::proposal(ctx, user.clone(), amount).await?;

    let mut oldmaid = Oldmaid::new(ctx.author());
    let mut other = UserId::new(u64::min_value());

    let (tx, mut rx) = mpsc::channel::<Signal>(5);

    loop {
        let timeout = oldmaid
            .turn_timeout
            .map(|duration| tokio::time::sleep(duration));
        let timeout = async {
            if let Some(sleep) = timeout {
                sleep.await;
                true
            } else {
                std::future::pending::<bool>().await
            }
        };

        let collector = ComponentInteractionCollector::new(ctx);

        let event = select! {
            _ = timeout => Event::Timeout,
            inter = collector.next() => Event::Interaction(inter.unwrap()),
            signal = rx.recv() => Event::Receiver(signal.unwrap()),
        };

        match event {
            Event::Receiver(signal) => match signal {
                Signal::GameEnd(inter, card) => {
                    let data = ctx.data();
                    message::end_game(ctx, &inter, &oldmaid, card).await?;

                    if let Some(amount) = amount {
                        let loser = oldmaid.get_player_with_oldmaid();
                        let loser = get_member(ctx, loser.id).await?;
                        let mut write = loser.write().await;

                        write.remove_balance(amount, &data.pool).await?;

                        let winner = oldmaid.get_winner();
                        let winner = get_member(ctx, winner.id).await?;
                        let mut write = winner.write().await;

                        write.add_balalance(amount, &data.pool).await?;
                    }

                    break;
                }
                Signal::StartTurn(inter, card) => {
                    oldmaid.reset_confirmed_card_index();
                    oldmaid.discard_pairs();
                    oldmaid.trigger_timeout();

                    if let Some(time) = oldmaid.message_timeout {
                        if time.elapsed().as_secs() < 15 {
                            message::in_choose_card(ctx, &inter, &oldmaid, card).await?;
                        } else {
                            oldmaid.trigger_message_timeout();
                            message::choose_card(ctx, &inter, &oldmaid, card).await?;
                        }
                    }

                    oldmaid.next_turn();
                }
                Signal::Timeout(_inter) => {}
            },
            Event::Interaction(inter) => {
                if inter.data.custom_id == format!("{}_resign", ctx.id()) {
                    if inter.user.id == other {
                        message::resign(ctx, &inter).await?;
                        break;
                    } else {
                        message::not_your_game(ctx, &inter).await?;
                    }
                }

                if inter.data.custom_id == format!("{}_accept", ctx.id()) {
                    if let Some(user) = user.clone() {
                        if inter.user.id == user.id {
                            if let Some(amount) = amount {
                                let member = get_member(ctx, user.id).await?;
                                let read = member.read().await;

                                if read.balance < amount {
                                    message::not_enough_balance(ctx, &inter).await?;
                                    return Ok(());
                                }
                            }

                            other = inter.user.id;

                            oldmaid.deal_cards();
                            oldmaid.discard_pairs();

                            message::first_turn(ctx, inter.clone(), &oldmaid).await?;

                            oldmaid.trigger_timeout();
                            oldmaid.trigger_message_timeout();
                        } else {
                            message::not_your_game(ctx, &inter).await?;
                        }
                    } else {
                        other = inter.user.id;
                        oldmaid.add_player(&inter.user)?;
                    }
                }

                if inter.data.custom_id == format!("{}_cards", ctx.id()) {
                    let player = oldmaid.get_rival();

                    if inter.user.id == player.id {
                        message::show_cards(ctx, &inter, player).await?;
                    } else {
                        message::not_your_turn(ctx, &inter).await?;
                    }
                }

                if let Some(index) = &inter
                    .data
                    .custom_id
                    .strip_prefix(&format!("{}_card_", ctx.id()))
                {
                    if inter.user.id == oldmaid.get_actual().id {
                        println!("actual {}", oldmaid.get_actual().name);
                        if let Ok(index) = index.parse::<usize>() {
                            let actual = oldmaid.players.first_mut().unwrap();

                            if let Some(confirmed_index) = actual.confirmed_card_index {
                                if index == confirmed_index {
                                    let rival = oldmaid.get_mut_rival();
                                    let taken_card = rival.take_card(index);

                                    if taken_card.clone().is_joker() {
                                        let current = oldmaid.players.first_mut().unwrap();
                                        current.hand.push(taken_card.clone());

                                        if oldmaid.cards_in_game() == 1 {
                                            tx.send(Signal::GameEnd(inter, taken_card)).await?;
                                        } else {
                                            tx.send(Signal::StartTurn(inter, taken_card)).await?;
                                        }
                                    } else {
                                        let current = oldmaid.players.first_mut().unwrap();
                                        current.discard_card(&taken_card);

                                        if oldmaid.cards_in_game() == 1 {
                                            tx.send(Signal::GameEnd(inter, taken_card)).await?;
                                        } else {
                                            tx.send(Signal::StartTurn(inter, taken_card)).await?;
                                        }
                                    }
                                } else {
                                    actual.confirmed_card_index = Some(index);
                                    message::just_update(
                                        ctx,
                                        inter.clone(),
                                        &oldmaid,
                                        inter.message.content,
                                    )
                                    .await?;
                                }
                            } else {
                                actual.confirmed_card_index = Some(index);
                                message::just_update(
                                    ctx,
                                    inter.clone(),
                                    &oldmaid,
                                    inter.message.content,
                                )
                                .await?;
                            }
                        }
                    } else {
                        message::not_your_turn(ctx, &inter).await?;
                    }
                }
            }
            Event::Timeout => {}
        }
    }

    Ok(())
}
