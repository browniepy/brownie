use crate::translation::translate;
use crate::{Context, Error};
use poise::serenity_prelude::User;
use poise::{
    serenity_prelude::{
        ButtonStyle, ComponentInteraction, CreateActionRow, CreateButton,
        CreateInteractionResponse, CreateInteractionResponseFollowup,
        CreateInteractionResponseMessage, Message,
    },
    CreateReply,
};
use types::{
    cards::poker::Card,
    oldmaid::{Oldmaid, Player},
};

async fn hand_buttons(
    ctx: Context<'_>,
    player: &Player,
    confirmed_index: Option<usize>,
) -> Vec<CreateActionRow> {
    let mut action_rows = Vec::new();
    let mut curr_buttons = Vec::new();

    for (i, _card) in player.hand.as_slice().iter().enumerate() {
        let style = if let Some(confirmed_index) = confirmed_index {
            if i == confirmed_index {
                ButtonStyle::Primary
            } else {
                ButtonStyle::Secondary
            }
        } else {
            ButtonStyle::Secondary
        };

        let button = CreateButton::new(format!("{}_card_{}", ctx.id(), i))
            .style(style)
            .label(format!("{}?", i));

        curr_buttons.push(button);

        if (i + 1) % 5 == 0 || i == player.hand.len() - 1 {
            action_rows.push(CreateActionRow::Buttons(curr_buttons.clone()));
            curr_buttons.clear();
        }
    }

    action_rows.push(CreateActionRow::Buttons(vec![CreateButton::new(format!(
        "{}_cards",
        ctx.id()
    ))
    .style(ButtonStyle::Secondary)
    .label("Mis cartas")]));

    action_rows
}

pub async fn first_turn(
    ctx: Context<'_>,
    inter: ComponentInteraction,
    oldmaid: &Oldmaid,
) -> Result<(), Error> {
    let rival = oldmaid.get_rival();
    let actual = oldmaid.get_actual();

    let joker_player = oldmaid.get_player_with_oldmaid();

    let comps = hand_buttons(ctx, rival, actual.confirmed_card_index).await;

    inter.create_response(ctx, CreateInteractionResponse::UpdateMessage(
            CreateInteractionResponseMessage::new()
            .content(translate!(ctx, "om-first-turn", userJoker: &joker_player.name, userA: &actual.name))
            .components(comps)
            )).await?;

    Ok(())
}

// update message with the end of the game
pub async fn end_game(
    ctx: Context<'_>,
    inter: &ComponentInteraction,
    oldmaid: &Oldmaid,
    last_card: Card,
) -> Result<(), Error> {
    let loser = oldmaid.get_player_with_oldmaid();
    let winner = oldmaid.get_winner();

    let last = format!("{:?}", last_card);

    inter
        .create_response(
            ctx,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .content(translate!(ctx, "om-end", winner: &winner.name, loser: &loser.name, card: last))
                    .components(vec![]),
            ),
        )
        .await?;

    Ok(())
}

pub async fn just_update(
    ctx: Context<'_>,
    inter: ComponentInteraction,
    oldmaid: &Oldmaid,
    content: String,
) -> Result<(), Error> {
    let rival = oldmaid.get_rival();
    let actual = oldmaid.get_actual();

    let comps = hand_buttons(ctx, rival, actual.confirmed_card_index).await;

    inter
        .create_response(
            ctx,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .content(content)
                    .components(comps),
            ),
        )
        .await?;

    Ok(())
}

// respond ephemeral with the user enumerate cards
pub async fn show_cards(
    ctx: Context<'_>,
    inter: &ComponentInteraction,
    player: &Player,
) -> Result<(), Error> {
    let mut cards = Vec::new();
    for (i, card) in player.hand.iter().enumerate() {
        cards.push(format!("{}? {:?} ", i, card));
    }
    inter
        .create_response(
            ctx,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(cards.join(", "))
                    .ephemeral(true),
            ),
        )
        .await?;
    Ok(())
}

// new turn message without sending a new message
pub async fn in_choose_card(
    ctx: Context<'_>,
    inter: &ComponentInteraction,
    oldmaid: &Oldmaid,
    last_card: Card,
) -> Result<(), Error> {
    let rival = oldmaid.get_rival();
    let actual = oldmaid.get_actual();

    let joker_player = oldmaid.get_player_with_oldmaid();

    let comps = hand_buttons(ctx, actual, rival.confirmed_card_index).await;

    let last = if last_card.is_joker() {
        "🃏 Joker".to_string()
    } else {
        format!("{:?}", last_card)
    };

    let pairs = oldmaid.discarded_pairs();
    let choose_msg =
        translate!(ctx, "om-choosed-card", user: &actual.name, card: last, pairs: pairs);

    let turn_msg = translate!(ctx, "om-turn", userJoker: &joker_player.name, userB: &actual.name, userA: &rival.name);

    inter
        .create_response(
            ctx,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .content(format!("{}\n\n{}", choose_msg, turn_msg))
                    .components(comps),
            ),
        )
        .await?;

    Ok(())
}

pub async fn not_your_game(ctx: Context<'_>, inter: &ComponentInteraction) -> Result<(), Error> {
    inter
        .create_response(
            ctx,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(translate!(ctx, "om-not-your-game"))
                    .ephemeral(true),
            ),
        )
        .await?;
    Ok(())
}

// not your turn message
pub async fn not_your_turn(ctx: Context<'_>, inter: &ComponentInteraction) -> Result<(), Error> {
    inter
        .create_response(
            ctx,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(translate!(ctx, "om-not-your-turn"))
                    .ephemeral(true),
            ),
        )
        .await?;
    Ok(())
}

// game resignted message
pub async fn resign(ctx: Context<'_>, inter: &ComponentInteraction) -> Result<(), Error> {
    inter
        .create_response(
            ctx,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .content(translate!(ctx, "resign"))
                    .components(vec![]),
            ),
        )
        .await?;
    Ok(())
}

// not enough balance to accept the game message
pub async fn not_enough_balance(
    ctx: Context<'_>,
    inter: &ComponentInteraction,
) -> Result<(), Error> {
    inter
        .create_response(
            ctx,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .content(translate!(ctx, "om-not-enough-balance"))
                    .components(vec![]),
            ),
        )
        .await?;
    Ok(())
}

pub async fn choose_card(
    ctx: Context<'_>,
    inter: &ComponentInteraction,
    oldmaid: &Oldmaid,
    last_card: Card,
) -> Result<Message, Error> {
    let rival = oldmaid.get_rival();
    let actual = oldmaid.get_actual();

    let joker_player = oldmaid.get_player_with_oldmaid();

    let comps = hand_buttons(ctx, actual, rival.confirmed_card_index).await;

    let last = if last_card.is_joker() {
        "🃏 Joker".to_string()
    } else {
        format!("{:?}", last_card)
    };

    let pairs = oldmaid.discarded_pairs();
    let choose_msg =
        translate!(ctx, "om-choosed-card", user: &actual.name, card: last, pairs: pairs);

    let turn_msg = translate!(ctx, "om-turn", userJoker: &joker_player.name, userB: &actual.name, userA: &rival.name);

    inter
        .create_response(
            ctx,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .content(choose_msg)
                    .components(vec![]),
            ),
        )
        .await?;

    let message = inter
        .create_followup(
            ctx,
            CreateInteractionResponseFollowup::new()
                .content(turn_msg)
                .components(comps),
        )
        .await?;

    Ok(message)
}

pub async fn proposal(
    ctx: Context<'_>,
    user: Option<User>,
    amount: Option<i32>,
) -> Result<Message, Error> {
    let bet = match amount {
        Some(amount) => translate!(ctx, "with-amount", amount: amount),
        None => translate!(ctx, "free-bet"),
    };

    let proposal = match user {
        Some(user) => translate!(ctx, "om-proposal", user: user.display_name()),
        None => translate!(ctx, "proposal"),
    };

    let message = ctx
        .send(
            CreateReply::default()
                .content(format!("{}\n{}", proposal, bet))
                .components(vec![CreateActionRow::Buttons(vec![
                    CreateButton::new(format!("{}_accept", ctx.id()))
                        .style(ButtonStyle::Secondary)
                        .label("Aceptar"),
                    CreateButton::new(format!("{}_resign", ctx.id()))
                        .style(ButtonStyle::Secondary)
                        .label("Resignar"),
                ])]),
        )
        .await?
        .into_message()
        .await?;

    Ok(message)
}
