use crate::{Context, Error};
use poise::{
    serenity_prelude::{
        ButtonStyle, ComponentInteraction, CreateActionRow, CreateButton,
        CreateInteractionResponse, CreateInteractionResponseFollowup,
        CreateInteractionResponseMessage, Message, MessageId,
    },
    CreateReply,
};
use types::bj::{Blackjack, State};

fn final_table_str(bj: &mut Blackjack) -> Result<String, Error> {
    let text = show_table_str(bj)?;

    let results = bj.get_results();

    let mut re = String::new();

    if let Some(winners) = results.winners {
        if winners.len() == 1 {
            let player = winners.first().unwrap();
            re.push_str(&format!("{} ganó la ronda", player.name));
        } else {
            let winners = winners
                .iter()
                .map(|p| p.name.clone())
                .collect::<Vec<String>>();
            re.push_str(&format!("{} ganaron", winners.join(", ")));
        }
    } else {
        re.push_str("Nadie ganó esta ronda");
    }

    re.push_str(&text);
    Ok(re)
}

fn show_table_str(bj: &mut Blackjack) -> Result<String, Error> {
    let dealer_val = if !bj.all_stand() {
        bj.dealer.hand_value(true)
    } else {
        bj.dealer.hand_value(false)
    };

    let dealer_hand = if !bj.all_stand() {
        bj.dealer.dbg_hand(true)
    } else {
        bj.dealer.dbg_hand(false)
    };

    let deck_text = format!("Cartas restantes {}\n\n", bj.deck.len());

    let state = if bj.is_dealer_bust { " Bust" } else { "" };

    let mut text = format!(
        "{}- Dealer{} {}\n{}\n\n",
        deck_text, state, dealer_val, dealer_hand
    );

    for player in bj.players.values() {
        let state = match player.state {
            State::None => "",
            State::Stand => " Stand",
            State::Bust => " Bust",
            State::Blackjack => " Blackjack",
        };

        text.push_str(&format!(
            "- {}{} {}\n{}\n\n",
            player.name,
            state,
            player.hand_value(),
            player.dbg_hand()
        ));
    }

    Ok(text)
}
pub struct Responses;

pub async fn comps_bj(ctx: Context<'_>, bj: &mut Blackjack) -> Vec<CreateActionRow> {
    let mut comps = vec![CreateActionRow::Buttons(vec![
        CreateButton::new(format!("{}_hit", ctx.id()))
            .style(ButtonStyle::Secondary)
            .label("Hit"),
        CreateButton::new(format!("{}_stand", ctx.id()))
            .style(ButtonStyle::Secondary)
            .label("Stand"),
        CreateButton::new(format!("{}_double", ctx.id()))
            .style(ButtonStyle::Secondary)
            .label("Double")
            .disabled(true),
    ])];

    if !bj.is_solo() {
        comps.push(CreateActionRow::Buttons(vec![
            CreateButton::new(format!("{}_leave", ctx.id()))
                .style(ButtonStyle::Secondary)
                .label("Leave"),
            CreateButton::new(format!("{}_join", ctx.id(),))
                .style(ButtonStyle::Secondary)
                .label("Join")
                .disabled(bj.is_full()),
        ]));
    }

    comps
}

impl Responses {
    pub async fn first(ctx: Context<'_>, bj: &mut Blackjack) -> Result<Message, Error> {
        let text = show_table_str(bj)?;

        let res = ctx
            .send(
                CreateReply::default()
                    .content(text)
                    .components(comps_bj(ctx, bj).await),
            )
            .await?
            .into_message()
            .await?;
        Ok(res)
    }

    pub async fn round_result(
        ctx: Context<'_>,
        bj: &mut Blackjack,
        inter: &ComponentInteraction,
    ) -> Result<(), Error> {
        let text = show_table_str(bj)?;

        inter
            .create_response(
                ctx,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new()
                        .content(text)
                        .components(comps_bj(ctx, bj).await),
                ),
            )
            .await?;

        Ok(())
    }

    pub async fn update(
        ctx: Context<'_>,
        bj: &mut Blackjack,
        inter: &ComponentInteraction,
    ) -> Result<(), Error> {
        let text = show_table_str(bj)?;

        inter
            .create_response(
                ctx,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new()
                        .content(text)
                        .components(if bj.all_stand() {
                            vec![]
                        } else {
                            comps_bj(ctx, bj).await
                        }),
                ),
            )
            .await?;

        Ok(())
    }

    pub async fn update_followup(
        ctx: Context<'_>,
        bj: &mut Blackjack,
        inter: &ComponentInteraction,
        msg: MessageId,
    ) -> Result<(), Error> {
        let text = if bj.dealer.hand_value(false) >= 17 {
            final_table_str(bj)?
        } else {
            show_table_str(bj)?
        };

        inter
            .edit_followup(
                ctx,
                msg,
                CreateInteractionResponseFollowup::new()
                    .content(text)
                    .components(vec![]),
            )
            .await?;

        Ok(())
    }

    pub async fn new_round(
        ctx: Context<'_>,
        bj: &mut Blackjack,
        inter: &ComponentInteraction,
    ) -> Result<Message, Error> {
        let text = show_table_str(bj)?;

        let res = inter
            .create_followup(
                ctx,
                CreateInteractionResponseFollowup::new()
                    .content(text)
                    .components(comps_bj(ctx, bj).await),
            )
            .await?;

        Ok(res)
    }
}
