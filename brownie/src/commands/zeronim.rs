use super::messages::nim;
use crate::{types::zeronim::Nim, Context, Error};
use poise::serenity_prelude::{ComponentInteraction, ComponentInteractionCollector, User};
use std::time::Duration;
use tokio::{select, sync::mpsc, time::sleep};

enum Event {
    Interaction(ComponentInteraction),
    Receiver(Signal),
}

enum Signal {
    RoundStart(ComponentInteraction),
    GameEnd(ComponentInteraction),
    RoundEnd(ComponentInteraction),
}

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "gambling"
)]
pub async fn nim(ctx: Context<'_>, user: User, amount: Option<i32>) -> Result<(), Error> {
    use crate::get_member;

    if let Some(amount) = amount {
        let member = get_member(ctx, ctx.author().id).await?;
        let read = member.read().await;

        if read.balance < amount {
            return Err("You don't have enough balance".into());
        }
    }

    let mut nim = Nim::new(ctx.author().clone(), user.clone());

    let (tx, mut rx) = mpsc::channel::<Signal>(1);

    let mut message = nim::proposal(ctx, user).await?;
    let mut ephemeral_message = None;

    loop {
        let collector = ComponentInteractionCollector::new(ctx);

        let event = select! {
            inter = collector.next() => Event::Interaction(inter.unwrap()),
            signal = rx.recv() => Event::Receiver(signal.unwrap()),
        };

        match event {
            Event::Receiver(signal) => match signal {
                Signal::RoundStart(inter) => {
                    nim.deal_cards();

                    message = nim::next_round(ctx, &nim, &inter).await?;
                }
                Signal::RoundEnd(inter) => {
                    nim.reset_state();

                    sleep(Duration::from_secs(10)).await;
                    tx.send(Signal::RoundStart(inter)).await?;
                }
                Signal::GameEnd(inter) => {
                    nim::game_end(ctx, &inter, &nim, message.id).await?;
                    break;
                }
            },
            Event::Interaction(inter) => {
                if inter.data.custom_id == format!("accept_{}", ctx.id()) {
                    nim.deal_cards();
                    nim::first_turn(ctx, &nim, &inter).await?;
                }

                if inter.data.custom_id == format!("decline_{}", ctx.id()) {
                    break;
                }

                if inter.data.custom_id == format!("choose_{}", ctx.id()) {
                    inter.defer_ephemeral(ctx).await?;
                    ephemeral_message = Some(nim::choose_card(ctx, &inter, &nim).await?);
                }

                if let Some(index) = &inter
                    .data
                    .custom_id
                    .strip_prefix(&format!("{}_card_", ctx.id()))
                {
                    if let Ok(index) = index.parse::<usize>() {
                        inter.defer(ctx).await?;

                        inter
                            .delete_followup(ctx, ephemeral_message.clone().unwrap().id)
                            .await?;

                        nim.put_card(index);

                        if nim.score > 9 {
                            let actual = nim.get_mut_rival();
                            actual.wins += 1;

                            if nim.finish() {
                                tx.send(Signal::GameEnd(inter.clone())).await?;
                            } else {
                                nim::round_lose(ctx, &inter, &nim, message.id).await?;
                                tx.send(Signal::RoundEnd(inter.clone())).await?;
                            }
                        } else {
                            nim.next_player();
                            nim::next_turn(ctx, &inter, &nim, message.id).await?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
