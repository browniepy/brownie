mod responses;

use crate::{get_member, Context, Error, Parse};
use poise::serenity_prelude::{ComponentInteraction, ComponentInteractionCollector};
use tokio::{
    sync::mpsc,
    time::{sleep, Duration},
};
use types::blackjack::*;

enum Reason {
    Timeout,
    Completed,
}

enum Signal {
    Finish {
        inter: ComponentInteraction,
        reason: Reason,
    },
    Tick,
}

enum Event {
    Interaction(ComponentInteraction),
    Receiver(Signal),
}

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "gambling"
)]
pub async fn blackjack(ctx: Context<'_>, amount: Option<String>) -> Result<(), Error> {
    let bet = Parse::amount(ctx, ctx.author().id, amount).await?;
    let mut last_inter = None;
    let mut bj = Blackjack::new(ctx.author().clone(), bet);

    let member = get_member(ctx, ctx.author().id).await?;
    let mut write = member.write().await;

    bj.set_timeout();
    bj.deal_cards(&mut write.deck);

    let (tx, mut rx) = mpsc::channel::<Signal>(5);
    let tx_clone = tx.clone();

    let msg = responses::first(ctx, &mut bj, &write.deck).await?;

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));

        loop {
            interval.tick().await;

            if tx_clone.send(Signal::Tick).await.is_err() {
                break;
            }
        }
    });

    loop {
        let collector = ComponentInteractionCollector::new(ctx);

        let event = tokio::select! {
            Some(inter) = collector.next() => Event::Interaction(inter),
            Some(signal) = rx.recv() => Event::Receiver(signal),
        };

        match event {
            Event::Receiver(signal) => match signal {
                Signal::Tick => {
                    println!("Tick");

                    if bj.timeout.is_some() {
                        bj.decrement_timeout();

                        if bj.is_timeout() {
                            tx.send(Signal::Finish {
                                inter: last_inter.clone().unwrap(),
                                reason: Reason::Timeout,
                            })
                            .await?;
                        }
                    }
                }

                Signal::Finish { inter, reason } => {
                    match reason {
                        Reason::Timeout => {}
                        Reason::Completed => {
                            responses::round_result(ctx, &mut bj, &inter, msg.id, &write.deck)
                                .await?;
                        }
                    }
                    return Ok(());
                }
            },
            Event::Interaction(inter) => {
                last_inter = Some(inter.clone());

                if inter.user.id == bj.player.id {
                    bj.set_timeout();

                    if inter.data.custom_id == format!("{}_hit", ctx.id()) {
                        bj.player_hit(&mut write.deck);
                        responses::update(ctx, &mut bj, &inter, &write.deck).await?;

                        if bj.player.is_bust() {
                            sleep(Duration::from_secs(1)).await;
                        }

                        let result = bj.round_result();
                        tx.send(Signal::Finish {
                            inter: inter.clone(),
                            reason: Reason::Completed,
                        })
                        .await?;
                    }

                    if inter.data.custom_id == format!("{}_stand", ctx.id()) {
                        bj.player.state = State::Stand;

                        responses::update(ctx, &mut bj, &inter, &write.deck).await?;
                        sleep(Duration::from_secs(1)).await;

                        while bj.dealer.hand_value(false) < 17 {
                            bj.dealer_hit(&mut write.deck);

                            responses::update_followup(ctx, &mut bj, &inter, msg.id, &write.deck)
                                .await?;
                            sleep(Duration::from_secs(1)).await;
                        }

                        let result = bj.round_result();
                        tx.send(Signal::Finish {
                            inter: inter.clone(),
                            reason: Reason::Completed,
                        })
                        .await?;
                    }
                } else {
                }
            }
        }
    }
}
