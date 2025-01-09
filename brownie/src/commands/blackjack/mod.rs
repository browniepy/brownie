mod responses;

use crate::{Context, Error, Parse};
use poise::serenity_prelude::{ComponentInteraction, ComponentInteractionCollector};
use tokio::{
    sync::mpsc,
    time::{sleep, Duration},
};
use types::blackjack::{Blackjack, FinishReason, State};

enum Signal {
    NextRound {
        inter: ComponentInteraction,
    },
    GameEnd {
        inter: ComponentInteraction,
        reason: FinishReason,
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

    bj.set_timeout();
    bj.deal_cards();

    let (tx, mut rx) = mpsc::channel::<Signal>(5);
    let tx_clone = tx.clone();

    let msg = responses::first(ctx, &mut bj).await?;

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
                            tx.send(Signal::GameEnd {
                                inter: last_inter.clone().unwrap(),
                                reason: FinishReason::Timeout,
                            })
                            .await?;
                        }
                    }
                }
                Signal::NextRound { inter } => {
                    let res = bj.round_result();
                    bj.set_last_event(res);

                    bj.clear_hands();
                    bj.deal_cards();
                    responses::new_round(ctx, &mut bj, &inter, msg.id).await?;
                }
                Signal::GameEnd { inter, reason } => {
                    println!("Game end");
                    rx.close();
                    return Ok(());
                }
            },
            Event::Interaction(inter) => {
                last_inter = Some(inter.clone());

                if inter.user.id == bj.player.id {
                    bj.set_timeout();
                    bj.check_deck();

                    if inter.data.custom_id == format!("{}_hit", ctx.id()) {
                        bj.player_hit();
                        responses::update(ctx, &mut bj, &inter).await?;

                        if bj.player.is_bust() {
                            sleep(Duration::from_secs(1)).await;

                            tx.send(Signal::NextRound {
                                inter: inter.clone(),
                            })
                            .await?;
                        }
                    }

                    if inter.data.custom_id == format!("{}_stand", ctx.id()) {
                        bj.player.state = State::Stand;

                        responses::update(ctx, &mut bj, &inter).await?;
                        sleep(Duration::from_secs(1)).await;

                        while bj.dealer.hand_value(false) < 17 {
                            bj.dealer_hit();

                            responses::update_followup(ctx, &mut bj, &inter, msg.id).await?;
                            sleep(Duration::from_secs(1)).await;
                        }

                        println!("Dealer hand value: {}", bj.dealer.hand_value(false));

                        tx.send(Signal::NextRound {
                            inter: inter.clone(),
                        })
                        .await?;
                    }
                } else {
                    // TODO: not your interaction message
                }
            }
        }
    }
}
