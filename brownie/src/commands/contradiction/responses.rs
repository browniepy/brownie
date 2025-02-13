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
use types::contradiction::{Contradiction, Level, Player, Reaction, Role};

use crate::{Context, Error};

pub fn bet_component(ctx: Context<'_>) -> Vec<CreateActionRow> {
    vec![CreateActionRow::Buttons(vec![CreateButton::new(format!(
        "{}_bet",
        ctx.id()
    ))
    .style(ButtonStyle::Secondary)
    .label("Apostar")])]
}

pub fn choose_component(ctx: Context<'_>) -> Vec<CreateActionRow> {
    vec![CreateActionRow::Buttons(vec![CreateButton::new(format!(
        "{}_choose",
        ctx.id()
    ))
    .style(ButtonStyle::Secondary)
    .label("Elegir objeto")])]
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
                    .label(shield.name());

                comps.push(button);
            }
        }
        Role::Attacker => {
            for (index, weapon) in contradict.weapons.iter().enumerate() {
                let button = CreateButton::new(format!("{}_object_{}", ctx.id(), index))
                    .style(ButtonStyle::Secondary)
                    .label(weapon.name());

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
    let content = "Primera ronda, primer turno\nElijan sus objetos".to_string();

    inter
        .create_response(
            ctx,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .content(content)
                    .components(choose_component(ctx)),
            ),
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
    let content = "Objetos elegidos\nHagan sus apuestas";

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
    let content = "usando los ultimos objetos restantes\nhagan sus apuestas";

    Ok(inter
        .create_followup(
            ctx,
            CreateInteractionResponseFollowup::new()
                .content(content)
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
                "Cuantos bios quieres apostar?",
                "amount",
            )
            .placeholder(format!("Tienes {} Bios", player.bios)),
        ),
    ]);

    inter
        .create_response(ctx, CreateInteractionResponse::Modal(modal))
        .await?;

    Ok(())
}

// new round
pub async fn new_round(
    ctx: Context<'_>,
    inter: &ModalInteraction,
    contradict: &Contradiction,
) -> Result<Message, Error> {
    let content = format!("ronda nro {}\nelijan sus objetos", contradict.round);

    Ok(inter
        .create_followup(
            ctx,
            CreateInteractionResponseFollowup::new()
                .content(content)
                .components(choose_component(ctx)),
        )
        .await?)
}

pub async fn new_turn(ctx: Context<'_>, inter: &ModalInteraction) -> Result<Message, Error> {
    let content = "nueva ronda elijan algo v:";

    Ok(inter
        .create_followup(
            ctx,
            CreateInteractionResponseFollowup::new()
                .content(content)
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
        Role::Defender => "Elige un escudo",
        Role::Attacker => "Elige un arma",
        Role::None => "error",
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

pub async fn comparison(
    ctx: Context<'_>,
    inter: &ModalInteraction,
    message: MessageId,
    contradict: &mut Contradiction,
    reaction: Reaction,
) -> Result<(), Error> {
    let player = contradict.less_bet_player();

    let content = match reaction {
        Reaction::Deviated => {
            format!("{} desvió la bala con escudo de hierro", player.name)
        }
        Reaction::Tased { shield, level } => {
            let level = match level {
                Level::Low => "leve",
                Level::Medium => "media",
                Level::High => "fuerte",
            };

            format!(
                "{} recibió una descarga eléctrica {} con escudo de {}",
                player.name,
                level,
                shield.name()
            )
        }
        Reaction::Stopped { weapon, shield } => {
            format!(
                "{} detuvo el ataque con {} usando escudo de {}",
                player.name,
                weapon.name(),
                shield.name()
            )
        }
        Reaction::Shot { shield } => {
            format!(
                "{} recibió un disparo con escudo de {}",
                player.name,
                shield.name()
            )
        }
        Reaction::Pierced { shield, level } => {
            let level = match level {
                Level::Low => "leve",
                Level::Medium => "media",
                Level::High => "fuerte",
            };

            format!(
                "{} fue cortado {} con escudo de {}",
                player.name,
                level,
                shield.name()
            )
        }
    };

    inter
        .edit_followup(
            ctx,
            message,
            CreateInteractionResponseFollowup::new()
                .content(content)
                .components(vec![]),
        )
        .await?;

    Ok(())
}

// join or accept messages
pub async fn accept(ctx: Context<'_>, user: &User, bet: i32) -> Result<Message, Error> {
    let content = format!(
        "Apuesta para {}\nDinero apostado {}",
        user.name.to_title_case(),
        bet
    );

    let components = vec![CreateActionRow::Buttons(vec![
        CreateButton::new(format!("{}_accept", ctx.id()))
            .style(ButtonStyle::Secondary)
            .label("Aceptar"),
        CreateButton::new(format!("{}_decline", ctx.id()))
            .style(ButtonStyle::Secondary)
            .label("Declinar"),
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
    let content = format!("Apuesta abierta\nDinero apostado {}", bet);

    let components = vec![CreateActionRow::Buttons(vec![CreateButton::new(format!(
        "{}_join",
        ctx.id()
    ))
    .style(ButtonStyle::Secondary)
    .label("Unirse")])];

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
    let content = "Apuesta rechazada";

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
