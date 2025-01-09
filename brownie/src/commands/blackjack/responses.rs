use crate::{Context, Error};
use poise::{
    serenity_prelude::{
        ButtonStyle, ComponentInteraction, CreateActionRow, CreateButton,
        CreateInteractionResponse, CreateInteractionResponseFollowup,
        CreateInteractionResponseMessage, Message, MessageId,
    },
    CreateReply,
};
use types::blackjack::Blackjack;

fn final_table_str(bj: &mut Blackjack) -> Result<String, Error> {
    use types::blackjack::RoundResult;
    let text = show_table_str(bj)?;
    let mut content = String::new();

    let event = bj.round_result();

    match event {
        RoundResult::Draw => {
            content.push_str("Empate\n");
        }
        RoundResult::Win { state } => {
            content.push_str("Ganaste la ronda \n");
        }
        RoundResult::Lose { bust } => {
            if bust {
                content.push_str("Te pasaste de 21\n");
            } else {
                content.push_str("Perdiste la ronda\n");
            }
        }
    }

    content.push_str(&text);
    Ok(content)
}

fn show_table_str(bj: &mut Blackjack) -> Result<String, Error> {
    use types::blackjack::{RoundResult, State};

    let dealer_val = if !bj.player.is_stand() {
        bj.dealer.hand_value(true)
    } else {
        bj.dealer.hand_value(false)
    };

    let dealer_hand = if !bj.player.is_stand() {
        bj.dealer.dbg_hand(true)
    } else {
        bj.dealer.dbg_hand(false)
    };

    let last_event_text = match &bj.last_event {
        Some(RoundResult::Draw) => "La ronda anterior fue un empate",
        Some(RoundResult::Win { state }) => match state {
            State::Blackjack => "Hiciste blackjack en la ronda anterior",
            _ => "Ganaste la ronda anterior",
        },
        Some(RoundResult::Lose { bust }) => {
            if *bust {
                "Te pasado de 21 en la ronda anterior"
            } else {
                "Perdiste la ronda anterior"
            }
        }
        None => "Primera ronda",
    };

    let deck_text = format!("Cartas restantes {}\n\n", bj.deck.len());

    let state = if bj.is_dealer_bust { " Bust" } else { "" };

    let mut text = format!(
        "{}\n{}- Dealer{} {}\n{}\n\n",
        last_event_text, deck_text, state, dealer_val, dealer_hand
    );

    let state = match bj.player.state {
        State::None => "",
        State::Stand => " Stand",
        State::Bust => " Bust",
        State::Blackjack => " Blackjack",
    };

    text.push_str(&format!(
        "- {}{} {}\n{}\n\n",
        bj.player.name,
        state,
        bj.player.hand_value(),
        bj.player.dbg_hand()
    ));

    Ok(text)
}

pub async fn comps_bj(ctx: Context<'_>, bj: &mut Blackjack) -> Vec<CreateActionRow> {
    let disabled = bj.player.is_stand() || bj.player.is_bust() || bj.player.is_blackjack();

    vec![CreateActionRow::Buttons(vec![
        CreateButton::new(format!("{}_hit", ctx.id()))
            .style(ButtonStyle::Secondary)
            .label("Hit")
            .disabled(disabled),
        CreateButton::new(format!("{}_stand", ctx.id()))
            .style(ButtonStyle::Secondary)
            .label("Stand")
            .disabled(disabled),
        CreateButton::new(format!("{}_double", ctx.id()))
            .style(ButtonStyle::Secondary)
            .label("Double")
            .disabled(disabled),
    ])]
}

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
                    .components(comps_bj(ctx, bj).await),
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
                .components(comps_bj(ctx, bj).await),
        )
        .await?;

    Ok(())
}

pub async fn new_round(
    ctx: Context<'_>,
    bj: &mut Blackjack,
    inter: &ComponentInteraction,
    msg: MessageId,
) -> Result<(), Error> {
    let text = show_table_str(bj)?;

    inter
        .edit_followup(
            ctx,
            msg,
            CreateInteractionResponseFollowup::new()
                .content(text)
                .components(comps_bj(ctx, bj).await),
        )
        .await?;

    Ok(())
}
