use inflector::Inflector;
use poise::{
    serenity_prelude::{
        ButtonStyle, ComponentInteraction, CreateActionRow, CreateButton, CreateInputText,
        CreateInteractionResponse, CreateInteractionResponseFollowup,
        CreateInteractionResponseMessage, CreateModal, InputTextStyle, Message, MessageId,
        ModalInteraction, User,
    },
    CreateReply,
};
use types::contradiction::{Contradiction, Player, Reaction, Role, Shield, Weapon};

use crate::{translation::translate, Context, Error};

pub fn bet_component(ctx: Context<'_>) -> Vec<CreateActionRow> {
    vec![CreateActionRow::Buttons(vec![CreateButton::new(format!(
        "{}_bet",
        ctx.id()
    ))
    .style(ButtonStyle::Secondary)
    .label(translate!(ctx, "bet"))])]
}

pub fn choose_component(ctx: Context<'_>) -> Vec<CreateActionRow> {
    vec![CreateActionRow::Buttons(vec![CreateButton::new(format!(
        "{}_choose",
        ctx.id()
    ))
    .style(ButtonStyle::Secondary)
    .label(translate!(ctx, "choose"))])]
}

pub fn objects_component(
    ctx: Context<'_>,
    inter: &ComponentInteraction,
    contradict: &Contradiction,
) -> Vec<CreateActionRow> {
    let mut comps = Vec::new();

    let player = contradict.get_player(inter.user.id).unwrap();

    match player.role {
        Role::Defender => {
            for (index, shield) in contradict.shields.iter().enumerate() {
                let button = CreateButton::new(format!("{}_object_{}", ctx.id(), index))
                    .style(ButtonStyle::Secondary)
                    .label(match shield {
                        Shield::Iron => translate!(ctx, "iron"),
                        Shield::Rubber => translate!(ctx, "rubber"),
                        Shield::Wood => translate!(ctx, "wood"),
                    });

                comps.push(button);
            }
        }
        Role::Attacker => {
            for (index, weapon) in contradict.weapons.iter().enumerate() {
                let button = CreateButton::new(format!("{}_object_{}", ctx.id(), index))
                    .style(ButtonStyle::Secondary)
                    .label(match weapon {
                        Weapon::Katana => translate!(ctx, "katana"),
                        Weapon::Taser => translate!(ctx, "taser"),
                        Weapon::Gun => translate!(ctx, "gun"),
                    });

                comps.push(button);
            }
        }
        Role::None => {}
    }

    vec![CreateActionRow::Buttons(comps)]
}

// game start
pub async fn start(
    ctx: Context<'_>,
    inter: &ComponentInteraction,
    contradict: &Contradiction,
) -> Result<(), Error> {
    let round_info = get_round(ctx, contradict);
    let content = translate!(ctx, "choose-phase");

    inter
        .create_response(
            ctx,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .content(format!("{}\n{}", round_info, content))
                    .components(choose_component(ctx)),
            ),
        )
        .await?;

    Ok(())
}

pub async fn final_result(
    ctx: Context<'_>,
    inter: &ModalInteraction,
    winner: &str,
    loser: &str,
    winner_message: Option<String>,
) -> Result<(), Error> {
    let mut content = translate!(ctx, "contradict-end", loser: loser, winner: winner);

    if let Some(winner_message) = winner_message {
        let message = translate!(ctx, "winner-message", message: winner_message);
        content.push_str(&format!("\n\n{}", message));
    }

    inter
        .create_followup(
            ctx,
            CreateInteractionResponseFollowup::new()
                .content(content)
                .components(vec![]),
        )
        .await?;

    Ok(())
}

// bet phase
pub async fn bet_phase(
    ctx: Context<'_>,
    inter: &ComponentInteraction,
    message: MessageId,
) -> Result<(), Error> {
    let content = translate!(ctx, "bet-phase");

    inter
        .edit_followup(
            ctx,
            message,
            CreateInteractionResponseFollowup::new()
                .content(content)
                .components(bet_component(ctx)),
        )
        .await?;

    Ok(())
}

pub async fn final_bet_phase(
    ctx: Context<'_>,
    inter: &ModalInteraction,
    contradict: &Contradiction,
) -> Result<Message, Error> {
    let round_info = get_round(ctx, contradict);
    let content = translate!(ctx, "last-round");

    Ok(inter
        .create_followup(
            ctx,
            CreateInteractionResponseFollowup::new()
                .content(format!("{}\n{}", round_info, content))
                .components(bet_component(ctx)),
        )
        .await?)
}

// bet modal
pub async fn bet_modal(
    ctx: Context<'_>,
    inter: &ComponentInteraction,
    player: &Player,
) -> Result<(), Error> {
    let modal = CreateModal::new(format!("{}_bet", ctx.id()), "Apuesta de Bios").components(vec![
        CreateActionRow::InputText(
            CreateInputText::new(
                InputTextStyle::Short,
                translate!(ctx, "how-many-bios"),
                "amount",
            )
            .placeholder(translate!(ctx, "your-bios", amount: player.bios)),
        ),
    ]);

    inter
        .create_response(ctx, CreateInteractionResponse::Modal(modal))
        .await?;

    Ok(())
}

pub fn get_round(ctx: Context<'_>, contradict: &Contradiction) -> String {
    let game = match contradict.round_info.game {
        1 => translate!(ctx, "fgame"),
        2 => translate!(ctx, "sgame"),
        3 => translate!(ctx, "tgame"),
        _ => translate!(ctx, "fogame"),
    };

    let round = match contradict.round_info.round {
        1 => translate!(ctx, "fround"),
        2 => translate!(ctx, "sround"),
        _ => translate!(ctx, "tround"),
    };

    translate!(ctx, "round-info", gnumber: game, rnumber: round)
}

// new round
pub async fn new_round(
    ctx: Context<'_>,
    inter: &ModalInteraction,
    contradict: &Contradiction,
) -> Result<Message, Error> {
    let round_info = get_round(ctx, contradict);
    let content = translate!(ctx, "choose-phase");

    Ok(inter
        .create_followup(
            ctx,
            CreateInteractionResponseFollowup::new()
                .content(format!("{}\n{}", round_info, content))
                .components(choose_component(ctx)),
        )
        .await?)
}

pub async fn new_turn(
    ctx: Context<'_>,
    inter: &ModalInteraction,
    contradict: &Contradiction,
) -> Result<Message, Error> {
    let round_info = get_round(ctx, contradict);
    let content = translate!(ctx, "choose-phase");

    Ok(inter
        .create_followup(
            ctx,
            CreateInteractionResponseFollowup::new()
                .content(format!("{}\n{}", round_info, content))
                .components(choose_component(ctx)),
        )
        .await?)
}

// choose weapon and shield
pub async fn choose_object(
    ctx: Context<'_>,
    inter: &ComponentInteraction,
    contradict: &Contradiction,
) -> Result<Message, Error> {
    let player = contradict.get_player(inter.user.id).unwrap();

    let content = match player.role {
        Role::Defender => translate!(ctx, "defender-choose"),
        Role::Attacker => translate!(ctx, "attacker-choose"),
        Role::None => "error".to_string(),
    };

    Ok(inter
        .create_followup(
            ctx,
            CreateInteractionResponseFollowup::new()
                .content(content)
                .components(objects_component(ctx, inter, contradict))
                .ephemeral(true),
        )
        .await?)
}

fn get_comparison_result(
    ctx: Context<'_>,
    defender: &str,
    attacker: &str,
    reaction: Reaction,
) -> String {
    match reaction {
        Reaction::Deviated => translate!(ctx, "gun-iron", defender: defender, attacker: attacker),

        Reaction::Tased { shield, level } => match shield {
            Shield::Iron => translate!(ctx, "taser-iron", defender: defender),
            _ => translate!(ctx, "taser-wood", defender: defender),
        },

        Reaction::Stopped { weapon, shield } => match (weapon, shield) {
            (Weapon::Katana, Shield::Iron) => translate!(ctx, "katana-iron", defender: defender),
            (Weapon::Taser, Shield::Rubber) => translate!(ctx, "taser-rubber", defender: defender),
            _ => String::from("None"),
        },

        Reaction::Shot { shield } => {
            let shield = match shield {
                Shield::Rubber => translate!(ctx, "rubber"),
                _ => translate!(ctx, "wood"),
            };

            translate!(ctx, "gun-wood-rubber", defender: defender, material: shield)
        }

        Reaction::Pierced { shield, level } => match shield {
            Shield::Rubber => translate!(ctx, "katana-rubber", defender: defender),
            Shield::Wood => translate!(ctx, "katana-wood", defender: defender),
            Shield::Iron => translate!(ctx, "katana-iron", defender: defender),
        },
    }
}

pub async fn comparison(
    ctx: Context<'_>,
    inter: &ModalInteraction,
    message: MessageId,
    contradict: &mut Contradiction,
    reaction: Reaction,
) -> Result<(), Error> {
    let defender = contradict.less_bet_player().name.clone();
    let attacker = contradict.greater_bet_player().name.clone();

    let content = get_comparison_result(ctx, &defender, &attacker, reaction);

    let a = contradict.players.first().unwrap();
    let b = contradict.players.last().unwrap();

    let sub_content = translate!(ctx, "bet-info", a: &a.name, aBios: a.current_bet, b: &b.name, bBios: b.current_bet);

    inter
        .edit_followup(
            ctx,
            message,
            CreateInteractionResponseFollowup::new()
                .content(format!("{}\n\n{}", content, sub_content))
                .components(vec![]),
        )
        .await?;

    Ok(())
}

// join or accept messages
pub async fn accept(ctx: Context<'_>, user: &User, bet: i32) -> Result<Message, Error> {
    let content = translate!(ctx, "contradict-proposal", user: user.name.to_title_case());

    let components = vec![CreateActionRow::Buttons(vec![
        CreateButton::new(format!("{}_accept", ctx.id()))
            .style(ButtonStyle::Secondary)
            .label(translate!(ctx, "accept")),
        CreateButton::new(format!("{}_decline", ctx.id()))
            .style(ButtonStyle::Secondary)
            .label(translate!(ctx, "decline")),
    ])];

    Ok(ctx
        .send(
            CreateReply::default()
                .content(content)
                .components(components),
        )
        .await?
        .into_message()
        .await?)
}

pub async fn join(ctx: Context<'_>, bet: i32) -> Result<Message, Error> {
    let content = translate!(ctx, "contradict-open");

    let components = vec![CreateActionRow::Buttons(vec![CreateButton::new(format!(
        "{}_join",
        ctx.id()
    ))
    .style(ButtonStyle::Secondary)
    .label(translate!(ctx, "join"))])];

    Ok(ctx
        .send(
            CreateReply::default()
                .content(content)
                .components(components),
        )
        .await?
        .into_message()
        .await?)
}

pub async fn declined(ctx: Context<'_>, inter: &ComponentInteraction) -> Result<(), Error> {
    let content = translate!(ctx, "declined-game");

    inter
        .create_response(
            ctx,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .content(content)
                    .components(vec![]),
            ),
        )
        .await?;

    Ok(())
}

// error messages
pub async fn bet_again(
    ctx: Context<'_>,
    inter: &ModalInteraction,
    message: MessageId,
    amount: usize,
) -> Result<(), Error> {
    let content = translate!(ctx, "bet-again", amount: amount);

    inter
        .edit_followup(
            ctx,
            message,
            CreateInteractionResponseFollowup::new().content(content),
        )
        .await?;

    Ok(())
}

pub async fn incorrect_bet(ctx: Context<'_>, inter: &ModalInteraction) -> Result<(), Error> {
    let content = translate!(ctx, "invalid-bet");

    inter
        .create_response(
            ctx,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(content)
                    .components(vec![])
                    .ephemeral(true),
            ),
        )
        .await?;

    Ok(())
}

pub async fn incorrect_inter(ctx: Context<'_>, inter: &ComponentInteraction) -> Result<(), Error> {
    let content = translate!(ctx, "not-for-you");

    inter
        .create_response(
            ctx,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .content(content)
                    .components(vec![])
                    .ephemeral(true),
            ),
        )
        .await?;

    Ok(())
}
