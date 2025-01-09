use super::{messages::dth, Context, Error};
use crate::{
    serenity::*,
    types::{DropCheck, DropRole, DropState, Dth, NearDeath},
};

use chrono::Utc;
use tokio::{
    select,
    sync::mpsc::channel,
    time::{interval, sleep, Duration},
};

enum Signal {
    StartRound(ComponentInteraction),
    Reanimate(ComponentInteraction),
    Timeout(ComponentInteraction),
    Tick,
}

enum Event {
    Interaction(ComponentInteraction),
    Receiver(Signal),
    Timeout,
}

#[poise::command(prefix_command, slash_command, category = "gambling", owners_only)]
pub async fn dth(ctx: Context<'_>, user: User) -> Result<(), Error> {
    let mut game = Dth::build(ctx.author().clone(), user.clone());
    let mut continue_inters = Vec::new();
    let mut strikes = 0;

    let (tx, mut rx) = channel::<Signal>(5);
    let tx_clone = tx.clone();

    let mut message = dth::proposal(ctx, &user.name).await?;
    let mut tick = true;

    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            tx_clone.send(Signal::Tick).await.unwrap();
        }
    });

    loop {
        let timeout = game.round_timeout.map(|duration| sleep(duration));
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
                Signal::Tick => {
                    if let Some(deadline) = game.deadline {
                        let remaining = deadline.signed_duration_since(Utc::now()).num_seconds();

                        if (4..=60).contains(&remaining) {
                            let content = if tick {
                                tick = false;
                                format!("tick {}", remaining)
                            } else {
                                tick = true;
                                format!("tock {}", remaining)
                            };

                            message
                                .edit(ctx, EditMessage::new().content(content))
                                .await?;
                            println!("tick {}", remaining);
                        } else if remaining <= 3 && remaining > 0 {
                            if remaining == 3 {
                                let content = if tick {
                                    tick = false;
                                    format!("tick {}", remaining)
                                } else {
                                    tick = true;
                                    format!("tock {}", remaining)
                                };

                                message
                                    .edit(ctx, EditMessage::new().content(content))
                                    .await?;
                                println!("tick {}", remaining);
                            } else if remaining == 1 {
                                message
                                    .edit(ctx, EditMessage::new().content("dong"))
                                    .await?;
                                println!("dong");
                            }
                        }
                    }
                    // hola
                }
                Signal::StartRound(inter) => {
                    game.swap_roles();
                    game.round_start_state();

                    message = dth::start_round(ctx, &inter, &game).await?;
                }
                Signal::Reanimate(inter) => {
                    dth::reanimation(ctx, &inter, &game).await?;
                    let checker = game.get_mut_player(DropRole::Checker);

                    match checker.inject_drug().await {
                        NearDeath::Death => {
                            dth::rcp_death(ctx, &inter, &checker.name).await?;
                            break;
                        }
                        NearDeath::Alive => {
                            dth::rcp_alive(ctx, &inter, &checker.name).await?;

                            let gclone = game.clone();
                            let tx_clone = tx.clone();

                            tokio::spawn(async move {
                                gclone.next_round_timer().await;
                                tx_clone.send(Signal::StartRound(inter)).await.unwrap();
                            });
                        }
                    }
                }
                Signal::Timeout(inter) => {
                    dth::round_expired(ctx, &inter, message.id).await?;
                }
            },
            Event::Interaction(inter) => match inter.data.custom_id.as_str() {
                id if id == format!("{}_continue", ctx.id()) => {
                    if game.players.contains_key(inter.user.id.as_ref()) {
                        game.set_inter(inter.clone());

                        if !continue_inters.contains(inter.user.id.as_ref()) {
                            continue_inters.push(inter.user.id);

                            if continue_inters.len() == 2 {
                                continue_inters.clear();

                                dth::continue_accept(ctx, &inter).await?;

                                let gclone = game.clone();
                                let tx_clone = tx.clone();

                                tokio::spawn(async move {
                                    gclone.next_round_timer().await;
                                    tx_clone.send(Signal::StartRound(inter)).await.unwrap();
                                });
                            } else {
                                inter
                                    .create_response(ctx, CreateInteractionResponse::Acknowledge)
                                    .await?;
                            }
                        } else {
                            dth::already_accept(ctx, &inter).await?;
                        }
                    } else {
                        dth::foreign_inter(ctx, &inter).await?;
                    }
                }
                id if id == format!("{}_accept", ctx.id()) => {
                    if true {
                        //if inter.user.id == user.id {
                        let start_date = format!("{}", game.next_round_date().format("%H:%M:%S"));
                        dth::game_accepted(ctx, &inter, &start_date).await?;

                        let gclone = game.clone();
                        let tx_clone = tx.clone();

                        tokio::spawn(async move {
                            gclone.next_round_timer().await;
                            tx_clone.send(Signal::StartRound(inter)).await.unwrap();
                        });
                    } else {
                        dth::foreign_inter(ctx, &inter).await?;
                    }
                }

                id if id == format!("{}_drop", ctx.id()) => {
                    let dropper = game.get_player(DropRole::Dropper);

                    if inter.user.id == dropper.id {
                        match game.state {
                            DropState::Hand => {
                                game.droph();
                                inter
                                    .create_response(ctx, CreateInteractionResponse::Acknowledge)
                                    .await?;
                            }
                            DropState::Dropped => {
                                dth::already_drop(ctx, &inter).await?;
                            }
                        };
                    } else {
                        dth::foreign_inter(ctx, &inter).await?;
                    }
                }

                id if id == format!("{}_check", ctx.id()) => {
                    let checker = game.get_player(DropRole::Checker);

                    if inter.user.id == checker.id {
                        match game.check() {
                            DropCheck::Failed => {
                                inter
                                    .create_response(ctx, CreateInteractionResponse::Acknowledge)
                                    .await?;
                                tx.send(Signal::Reanimate(inter)).await?;
                            }
                            DropCheck::Sucess(time) => {
                                let checker = game.get_player(DropRole::Checker);

                                dth::check_success(ctx, &inter, &checker.name, time).await?;

                                let gclone = game.clone();
                                let tx_clone = tx.clone();

                                tokio::spawn(async move {
                                    gclone.next_round_timer().await;
                                    tx_clone.send(Signal::StartRound(inter)).await.unwrap();
                                });
                            }
                        }
                    } else {
                        dth::foreign_inter(ctx, &inter).await?;
                    }
                }

                id if id == format!("{}_stats", ctx.id()) => {
                    dth::stats(ctx, &inter, &game).await?;
                }
                _ => {
                    continue;
                }
            },
            Event::Timeout => {
                match game.state {
                    DropState::Dropped => {
                        // c comprueba automaticamente
                        if let DropCheck::Sucess(time) = game.check() {
                            let checker = game.get_player(DropRole::Checker);
                            dth::check_success_msg(ctx, &mut message, &checker.name, time).await?;
                        }

                        let tx_clone = tx.clone();
                        let gclone = game.clone();

                        tokio::spawn(async move {
                            gclone.next_round_timer().await;
                            tx_clone
                                .send(Signal::StartRound(gclone.last_inter.unwrap()))
                                .await
                                .unwrap();
                        });
                    }
                    DropState::Hand => {
                        // añadir strike
                        strikes += 1;

                        if strikes >= 2 {
                            // acabar juego
                            return Ok(());
                        }
                    }
                }

                tx.send(Signal::Timeout(game.clone().last_inter.unwrap()))
                    .await
                    .unwrap();
            }
        }
    }

    Ok(())
}
