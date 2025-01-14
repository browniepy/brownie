use std::{cmp::Ordering, collections::HashMap, time::Duration, usize};

use crate::{mpsc_data::AirData, Context, Error};
use poise::{
    serenity_prelude::{
        ButtonStyle, ComponentInteraction, ComponentInteractionCollector, CreateActionRow,
        CreateButton, CreateInputText, CreateInteractionResponse,
        CreateInteractionResponseFollowup, CreateInteractionResponseMessage, CreateModal,
        InputTextStyle, ModalInteraction, ModalInteractionCollector, User, UserId,
    },
    CreateReply, Modal,
};
use tokio::{select, sync::mpsc::channel, time::interval};
use tracing::warn;
use types::{
    airpoker::{AirPoker, Player},
    cards::poker::{Card, PokerValue},
    evaluate::compare_hands,
    EvaluatePoker,
};

#[derive(poise::Modal)]
struct BetModal {
    amount: String,
}

enum Signal {
    SelectCardRound { inter: ComponentInteraction },
    BetRound { inter: ComponentInteraction },
    GameEnd { inter: ComponentInteraction },
    BetRoundEnd,
    SelectCardEnd,
    Tick,
}

enum Event {
    Interaction(ComponentInteraction),
    Receiver(Signal),
    ModalInter(ModalInteraction),
}

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "gambling"
)]
pub async fn airpoker(ctx: Context<'_>, user: User) -> Result<(), Error> {
    let mut channels = HashMap::new();

    let (tx, mut rx) = channel::<Signal>(15);
    let tx_clone = tx.clone();

    let mut message = ctx
        .send(CreateReply::default().content("airpoker").components(vec![
            CreateActionRow::Buttons(vec![
                CreateButton::new(format!("{}_accept", ctx.id()))
                    .label("Aceptar")
                    .style(ButtonStyle::Secondary),
                CreateButton::new(format!("{}_decline", ctx.id()))
                    .label("Resignar")
                    .style(ButtonStyle::Secondary),
        ]),
        ]))
        .await?
        .into_message()
        .await?;

    let data = AirData {
        sender: tx.clone(),
        airpoker: AirPoker::new(Player::new(ctx.author().clone()), Player::new(user.clone())),
        ok_selected: Vec::new(),
        ok_bet: Vec::new(),
        last_inter: None,
        first_round: true,
        message: message.id,
    };

    channels.insert(ctx.id(), data);

    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(1));

        loop {
            interval.tick().await;
            if tx_clone.send(Signal::Tick).await.is_err() {
                warn!("sender close for");
            }
        }
    });

    loop {
        let collector = ComponentInteractionCollector::new(ctx);
        let modal_collector = ModalInteractionCollector::new(ctx);

        let event = select! {
            inter = collector.next() => Event::Interaction(inter.unwrap()),
            signal = rx.recv() => Event::Receiver(signal.unwrap()),
            modal_inter = modal_collector.next() => Event::ModalInter(modal_inter.unwrap()),
        };

        match event {
            Event::Receiver(signal) => match signal {
                Signal::Tick => {
                    for (id, channel) in &mut channels {
                        handle_tick(&mut channel.airpoker).await;

                        if channel.airpoker.bet_timeout.is_some() {
                            if channel.airpoker.is_bet_timeout() {
                                channel.airpoker.delete_bet_timeout();
                                tx.send(Signal::BetRoundEnd).await?;
                            } else {
                                channel.airpoker.decrement_bet_timeout();
                            }
                        }

                        if channel.airpoker.select_card_timeout.is_some() {
                            if channel.airpoker.is_select_card_timeout() {
                                channel.airpoker.delete_select_card_timeout();
                                tx.send(Signal::SelectCardEnd).await?;
                            } else {
                                channel.airpoker.decrement_select_card_timeout();
                            }
                        }
                    }
                }

                Signal::GameEnd { inter } => {
                    if let Some(channel) = channels.get(&ctx.id()) {
                        let winner = channel
                            .airpoker
                            .players
                            .iter()
                            .find(|p| p.is_alive)
                            .unwrap();

                        return Ok(());
                    }
                }

                Signal::BetRound { inter } => {
                    if let Some(channel) = channels.get_mut(&ctx.id()) {
                        channel.airpoker.set_players_blind();
                        channel.airpoker.set_bet_timeout();

                        if channel.first_round {
                            inter
                                .create_response(
                                    ctx,
                                    CreateInteractionResponse::UpdateMessage(
                                        CreateInteractionResponseMessage::new()
                                            .content("apuesten")
                                            .components(vec![
                                                CreateActionRow::Buttons(vec![
                                                    CreateButton::new(format!("{}_bet", ctx.id()))
                                                        .label("Aumentar")
                                                        .style(ButtonStyle::Secondary),
                                                    CreateButton::new(format!("{}_call", ctx.id()))
                                                        .label("Igualar")
                                                        .style(ButtonStyle::Secondary),
                                                    CreateButton::new(format!("{}_fold", ctx.id()))
                                                        .label("Retirarse")
                                                        .style(ButtonStyle::Secondary),
                                                ]),
                                                CreateActionRow::Buttons(vec![CreateButton::new(
                                                    format!("{}_cards", ctx.id()),
                                                )
                                                .label("Mis cartas")
                                                .style(ButtonStyle::Secondary)]),
                                            ]),
                                    ),
                                )
                                .await?;
                        } else {
                            inter
                                .create_response(
                                    ctx,
                                    CreateInteractionResponse::Message(
                                        CreateInteractionResponseMessage::new()
                                            .content("apuesten")
                                            .components(vec![
                                                CreateActionRow::Buttons(vec![
                                                    CreateButton::new(format!("{}_bet", ctx.id()))
                                                        .label("Aumentar")
                                                        .style(ButtonStyle::Secondary),
                                                    CreateButton::new(format!("{}_call", ctx.id()))
                                                        .label("Igualar")
                                                        .style(ButtonStyle::Secondary),
                                                    CreateButton::new(format!("{}_fold", ctx.id()))
                                                        .label("Retirarse")
                                                        .style(ButtonStyle::Secondary),
                                                ]),
                                                CreateActionRow::Buttons(vec![CreateButton::new(
                                                    format!("{}_cards", ctx.id()),
                                                )
                                                .label("Mis cartas")
                                                .style(ButtonStyle::Secondary)]),
                                            ]),
                                    ),
                                )
                                .await?;
                        }
                    }
                }

                Signal::SelectCardRound { inter } => {
                    if let Some(channel) = channels.get_mut(&ctx.id()) {
                        channel.airpoker.set_select_card_timeout();

                        inter
                            .edit_followup(
                                ctx,
                                message.id,
                                CreateInteractionResponseFollowup::new()
                                    .content("elijan carta papus")
                                    .components(vec![CreateActionRow::Buttons(vec![
                                        CreateButton::new(format!("{}_choose", ctx.id()))
                                            .style(ButtonStyle::Secondary)
                                            .label("Elegir"),
                                    ])]),
                            )
                            .await?;
                    }
                }

                Signal::BetRoundEnd => {
                    if let Some(channel) = channels.get_mut(&ctx.id()) {
                        channel.airpoker.delete_bet_timeout();

                        tx.send(Signal::SelectCardRound {
                            inter: channel.last_inter.clone().unwrap(),
                        })
                        .await?;
                    }
                }

                Signal::SelectCardEnd => {
                    if let Some(channel) = channels.get_mut(&ctx.id()) {
                        if channel.ok_selected.len() == 1 {
                            let player = channel.airpoker.find_not_selected()?;
                            player.select_random_card();

                            let inter = channel.last_inter.clone().unwrap();

                            let p1 = channel.airpoker.players.first().unwrap();
                            let p2 = channel.airpoker.players.last().unwrap();

                            let a = match compare_hands(
                                &p1.selected_card.clone().unwrap().hand,
                                &p2.selected_card.clone().unwrap().hand,
                            ) {
                                Ordering::Less => p2,
                                Ordering::Greater => p1,
                                _ => return Err(":v".into()),
                            };

                            let b = Card::evaluate_hand(&a.selected_card.clone().unwrap().hand);

                            let c = Card::evaluate_hand(&p1.selected_card.clone().unwrap().hand);

                            let d = Card::evaluate_hand(&p2.selected_card.clone().unwrap().hand);

                            inter
                                .edit_followup(
                                    ctx,
                                    message.id,
                                    CreateInteractionResponseFollowup::new()
                                        .content(format!(
                                            "{} {:?} vs {} {:?}\n{} {:?} vs {} {:?}",
                                            p1.name,
                                            p1.selected_card,
                                            p2.name,
                                            p2.selected_card,
                                            p1.name,
                                            c,
                                            p2.name,
                                            d
                                        ))
                                        .components(vec![]),
                                )
                                .await?;
                        } else if channel.ok_selected.is_empty() {
                            channel
                                .sender
                                .send(Signal::GameEnd {
                                    inter: channel.last_inter.clone().unwrap(),
                                })
                                .await?;
                        } else {
                            let inter = channel.last_inter.clone().unwrap();

                            inter
                                .edit_followup(
                                    ctx,
                                    message.id,
                                    CreateInteractionResponseFollowup::new()
                                        .content("cartas seleccionadas")
                                        .components(vec![]),
                                )
                                .await?;

                            channel.ok_selected.clear();
                        }
                    }
                }
            },

            Event::Interaction(inter) => {
                if let Some(channel) = channels.get_mut(&ctx.id()) {
                    if inter.data.custom_id == format!("{}_accept", ctx.id()) {
                        channel.last_inter = Some(inter.clone());
                        channel.airpoker.deal_cards();

                        channel
                            .sender
                            .send(Signal::BetRound {
                                inter: inter.clone(),
                            })
                            .await?;
                    }

                    if inter.data.custom_id == format!("{}_decline", ctx.id()) {
                        break;
                    }

                    if inter.data.custom_id == format!("{}_bet", ctx.id()) {
                        let player = channel.airpoker.get_player(inter.user.id);

                        if let Ok(player) = player {
                            // check if player has sufficient bios to bet
                            if player.get_betable_air_bios() < channel.airpoker.blind as usize {
                                // do something
                            } else if !channel.ok_bet.contains(&inter.user.id) {
                                inter
                                    .create_response(
                                        ctx,
                                        CreateInteractionResponse::Modal(
                                            CreateModal::new(
                                                format!("{}_bet_modal", ctx.id()),
                                                "Apuesta",
                                            )
                                            .components(vec![CreateActionRow::InputText(
                                                CreateInputText::new(
                                                    InputTextStyle::Short,
                                                    "Cantidad de bios para apostar",
                                                    "amount",
                                                )
                                                .placeholder(format!(
                                                    "Tienes {} bios",
                                                    player.get_betable_air_bios()
                                                        - player.bet as usize
                                                )),
                                            )]),
                                        ),
                                    )
                                    .await?;
                            } else {
                                inter
                                    .create_response(ctx, CreateInteractionResponse::Acknowledge)
                                    .await?;
                            }
                        }
                    }

                    if inter.data.custom_id == format!("{}_cards", ctx.id()) {
                        channel.last_inter = Some(inter.clone());

                        let player = channel.airpoker.get_player(inter.user.id);

                        if let Ok(player) = player {
                            // hand with the sum of every card
                            let hand = player
                                .hand
                                .iter()
                                .map(|c| c.hand.iter().map(|c| c.value()).sum::<u8>())
                                .collect::<Vec<u8>>();
                            // convert into a str vec
                            let hand_str =
                                hand.iter().map(|c| c.to_string()).collect::<Vec<String>>();

                            inter
                                .create_response(
                                    ctx,
                                    CreateInteractionResponse::Message(
                                        CreateInteractionResponseMessage::new()
                                            .content(format!("Tus cartas son: {:?}", hand_str))
                                            .ephemeral(true),
                                    ),
                                )
                                .await?;
                        }
                    }

                    if inter.data.custom_id == format!("{}_choose", ctx.id()) {
                        channel.last_inter = Some(inter.clone());

                        inter.defer_ephemeral(ctx).await?;

                        let p = channel.airpoker.get_mut_player(inter.user.id)?;
                        let mut comps = Vec::new();

                        for (index, card) in p.hand.iter().enumerate() {
                            let button = CreateButton::new(format!("{}_card_{}", ctx.id(), index))
                                .style(ButtonStyle::Secondary)
                                .label(card.value().to_string());

                            comps.push(button);
                        }

                        let msg = inter
                            .create_followup(
                                ctx,
                                CreateInteractionResponseFollowup::new()
                                    .content("tus cartas")
                                    .components(vec![CreateActionRow::Buttons(comps)]),
                            )
                            .await?;

                        p.set_ephemeral(msg);
                    }

                    if let Some(index) = &inter
                        .data
                        .custom_id
                        .strip_prefix(&format!("{}_card_", ctx.id()))
                    {
                        if let Ok(index) = index.parse::<usize>() {
                            inter.defer(ctx).await?;

                            let player = channel.airpoker.get_mut_player(inter.user.id)?;

                            player.select_card(index);
                            channel.ok_selected.push(inter.user.id);

                            let ephemeral = player.get_ephemeral()?;

                            inter.delete_followup(ctx, ephemeral.id).await?;
                        }
                    }
                }
            }

            Event::ModalInter(modal_inter) => {
                if let Some(channel) = channels.get_mut(&ctx.id()) {
                    if modal_inter.data.custom_id == format!("{}_bet_modal", ctx.id()) {
                        let data = BetModal::parse(modal_inter.clone().data)?;

                        // parse amount to u8
                        let amount = data.amount.parse::<usize>();

                        let player = channel.airpoker.get_mut_player(modal_inter.user.id);

                        if let Ok(player) = player {
                            if let Ok(amount) = amount {
                                if amount > (player.get_betable_air_bios() - player.bet as usize) {
                                    modal_inter
                                        .create_response(
                                            ctx,
                                            CreateInteractionResponse::Message(
                                                CreateInteractionResponseMessage::new()
                                                    .ephemeral(true)
                                                    .content("error"),
                                            ),
                                        )
                                        .await?;
                                } else {
                                    player.bet = amount as u8;
                                    channel.ok_bet.push(modal_inter.user.id);
                                }

                                if channel.ok_bet.len() == 2 {
                                    channel.ok_bet.clear();
                                    channel
                                        .sender
                                        .send(Signal::SelectCardRound {
                                            inter: channel.last_inter.clone().unwrap(),
                                        })
                                        .await?;
                                } else {
                                    modal_inter
                                        .create_response(
                                            ctx,
                                            CreateInteractionResponse::Acknowledge,
                                        )
                                        .await?;
                                }
                            } else {
                                // do something
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

async fn handle_tick(airpoker: &mut AirPoker) {
    for player in airpoker.players.iter_mut() {
        if player.is_alive {
            let has_oxygen = player.consume_air_bio();
            if !has_oxygen {
                println!("{} se quedo sin oxigeno", player.name);
            } else if player.get_active_tank_duration() <= 5 {
                println!("{} se esta quedando sin oxigeno", player.name);
            }
        }
    }
}
