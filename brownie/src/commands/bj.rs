use crate::commands::messages::bj::Responses;
use crate::{Context, Error, Parse};
use poise::serenity_prelude::{ComponentInteraction, ComponentInteractionCollector};
use tokio::{
    sync::mpsc,
    time::{sleep, Duration},
};
use types::bj::{Blackjack, FinishReason, State};

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
pub async fn blackjack(ctx: Context<'_>, amount: String, solo: Option<bool>) -> Result<(), Error> {
    let bet = Parse::amount(ctx, ctx.author().id, Some(amount)).await?;

    let mut bj = Blackjack::new(
        ctx.author().clone(),
        bet,
        if solo.unwrap_or(true) { 1 } else { 2 },
    );

    bj.set_timeout();
    bj.deal_cards();

    // channel to receive signals from the event loop
    let (tx, mut rx) = mpsc::channel::<Signal>(5);
    let tx_clone = tx.clone();

    let mut last_inter = None;

    let mut msg = Responses::first(ctx, &mut bj).await?;

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));

        loop {
            interval.tick().await;
            if !tx_clone.is_closed() {
                tx_clone.send(Signal::Tick).await.unwrap();
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
                    println!("Next round");
                    bj.clear_hands();
                    bj.deal_cards();
                    msg = Responses::new_round(ctx, &mut bj, &inter).await?;
                }
                Signal::GameEnd { inter, reason } => {
                    println!("Game end");
                    rx.close();
                    return Ok(());
                }
            },
            Event::Interaction(inter) => {
                last_inter = Some(inter.clone());

                if bj.is_player_in_game(&inter.user) {
                    bj.set_timeout();
                    bj.check_deck();
                }

                if inter.data.custom_id == format!("{}_hit", ctx.id()) {
                    if let Some(player) = bj.get_player_mut(&inter.user) {
                        if player.can_hit() {
                            bj.player_hit(&inter.user);
                            Responses::update(ctx, &mut bj, &inter).await?;

                            if bj.all_bust() {
                                tx.send(Signal::NextRound {
                                    inter: inter.clone(),
                                })
                                .await?;
                            }
                        }
                    } else {
                        // TODO:
                    }
                }

                if inter.data.custom_id == format!("{}_stand", ctx.id()) {
                    if let Some(player) = bj.get_player_mut(&inter.user) {
                        if player.can_stand() {
                            player.state = State::Stand;

                            if bj.all_stand() {
                                Responses::update(ctx, &mut bj, &inter).await?;
                                sleep(Duration::from_secs(1)).await;

                                while bj.dealer.hand_value(false) < 17 {
                                    bj.dealer_hit();

                                    Responses::update_followup(ctx, &mut bj, &inter, msg.id)
                                        .await?;
                                    sleep(Duration::from_secs(1)).await;
                                }

                                println!("Dealer hand value: {}", bj.dealer.hand_value(false));

                                tx.send(Signal::NextRound {
                                    inter: inter.clone(),
                                })
                                .await?;
                            } else {
                                Responses::update(ctx, &mut bj, &inter).await?;
                            }
                        }
                    }
                }

                if inter.data.custom_id == format!("{}_leave", ctx.id()) {
                    if let Some(player) = bj.get_player(&inter.user) {
                        bj.remove_player(&inter.user);

                        if bj.players.is_empty() {
                            break;
                        }
                    } else {
                        // TODO:
                    }
                }
            }
        }
    }

    Ok(())
}
