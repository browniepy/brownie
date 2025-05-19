use super::super::CommonButton;
use crate::{translate, Context, Error, Parser};
use poise::{
    serenity_prelude::{
        ButtonStyle, ComponentInteraction, CreateActionRow, CreateButton, CreateInputText,
        CreateInteractionResponse, CreateInteractionResponseFollowup,
        CreateInteractionResponseMessage, CreateModal, InputTextStyle, Mentionable, Message,
        MessageId, ModalInteraction, User,
    },
    CreateReply,
};
use types::contradiction::{Contradiction, Player, Reaction, Role, ShieldEnum, WeaponEnum};

struct Button;

pub struct Response;

pub struct ModalRes;

impl ModalRes {
    pub async fn bet(
        ctx: Context<'_>,
        inter: &ComponentInteraction,
        player: &Player,
    ) -> Result<(), Error> {
        let your_bios = if player.bios > 0 {
            translate!(ctx, "your-bios", amount: Parser::num_with_commas(player.bios as i64))
        } else {
            translate!(ctx, "empty-bios")
        };

        let modal = CreateModal::new(
            format!("{}_bet", ctx.id()),
            translate!(ctx, "bet-modal-title"),
        )
        .components(vec![CreateActionRow::InputText(
            CreateInputText::new(
                InputTextStyle::Short,
                translate!(ctx, "how-many-bios"),
                "bios",
            )
            .required(false)
            .placeholder(your_bios),
        )]);

        inter
            .create_response(ctx, CreateInteractionResponse::Modal(modal))
            .await?;

        Ok(())
    }
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

    translate!(ctx, "contradict-round-info", gnumber: game, rnumber: round)
}

impl Response {
    pub async fn final_result(
        ctx: Context<'_>,
        inter: &ModalInteraction,
        winner: &str,
        loser: &str,
    ) -> Result<(), Error> {
        let content = translate!(ctx, "contradict-end", loser: loser, winner: winner);

        inter
            .create_followup(
                ctx,
                CreateInteractionResponseFollowup::new()
                    .content(content)
                    .allowed_mentions(crate::mentions())
                    .components(vec![]),
            )
            .await?;

        Ok(())
    }

    pub async fn new_round(
        ctx: Context<'_>,
        inter: &ModalInteraction,
        contradict: &Contradiction,
    ) -> Result<Message, Error> {
        let round_info = get_round(ctx, contradict);
        let content = translate!(ctx, "contradict-choose-phase");

        let message = inter
            .create_followup(
                ctx,
                CreateInteractionResponseFollowup::new()
                    .allowed_mentions(crate::mentions())
                    .components(Button::display_buttons(ctx, false, true))
                    .content(format!("{}\n{}", round_info, content)),
            )
            .await?;

        Ok(message)
    }

    pub async fn comparison(
        ctx: Context<'_>,
        inter: &ModalInteraction,
        message_id: MessageId,
        contradict: &mut Contradiction,
        reaction: Reaction,
    ) -> Result<(), Error> {
        let defender = contradict.less_bet_player().name.clone();
        let attacker = contradict.greater_bet_player().name.clone();

        let content = Self::get_comparison_result(ctx, &defender, &attacker, reaction);

        let a = contradict.players.first().unwrap();
        let b = contradict.players.last().unwrap();

        let sub_content = translate!(ctx, "contradict-bet-info", a: &a.name, aBios: Parser::num_with_commas(a.current_bet as i64), b: &b.name, bBios: Parser::num_with_commas(b.current_bet as i64));

        inter
            .edit_followup(
                ctx,
                message_id,
                CreateInteractionResponseFollowup::new()
                    .allowed_mentions(crate::mentions())
                    .content(format!("{}\n{}", sub_content, content))
                    .components(Button::display_buttons(ctx, true, true)),
            )
            .await?;

        Ok(())
    }

    fn get_comparison_result(
        ctx: Context<'_>,
        defender: &str,
        attacker: &str,
        reaction: Reaction,
    ) -> String {
        match reaction {
            Reaction::Deviated => {
                translate!(ctx, "gun-iron", defender: defender, attacker: attacker)
            }

            Reaction::Tased { shield, .. } => match shield {
                ShieldEnum::Iron => translate!(ctx, "taser-iron", defender: defender),
                _ => translate!(ctx, "taser-wood", defender: defender),
            },

            Reaction::Stopped { weapon, shield } => match (weapon, shield) {
                (WeaponEnum::Katana, ShieldEnum::Iron) => {
                    translate!(ctx, "katana-iron", defender: defender)
                }
                (WeaponEnum::Taser, ShieldEnum::Rubber) => {
                    translate!(ctx, "taser-rubber", defender: defender)
                }
                _ => String::from("None"),
            },

            Reaction::Shot { shield } => {
                let shield = match shield {
                    ShieldEnum::Rubber => translate!(ctx, "rubber"),
                    _ => translate!(ctx, "wood"),
                };

                translate!(ctx, "gun-wood-rubber", defender: defender, material: shield)
            }

            Reaction::Pierced { shield, .. } => match shield {
                ShieldEnum::Rubber => translate!(ctx, "katana-rubber", defender: defender),
                ShieldEnum::Wood => translate!(ctx, "katana-wood", defender: defender),
                ShieldEnum::Iron => translate!(ctx, "katana-iron", defender: defender),
            },
        }
    }

    pub async fn bet_draw(
        ctx: Context<'_>,
        inter: &ModalInteraction,
        message_id: MessageId,
    ) -> Result<(), Error> {
        let content = translate!(ctx, "contradict-bet-draw");

        inter
            .edit_followup(
                ctx,
                message_id,
                CreateInteractionResponseFollowup::new()
                    .content(content)
                    .components(Button::display_buttons(ctx, true, false)),
            )
            .await?;

        Ok(())
    }

    pub async fn request(ctx: Context<'_>, user: &User, bios: i64) -> Result<Message, Error> {
        let abbreviate = Parser::abbreviate_number(bios);
        let content = translate!(ctx, "contradict-request", user: user.mention().to_string(), bios: abbreviate);

        let message = ctx
            .send(
                CreateReply::default()
                    .content(content)
                    .allowed_mentions(crate::mentions())
                    .components(CommonButton::accept_or_decline(ctx, false)),
            )
            .await?
            .into_message()
            .await?;

        Ok(message)
    }

    pub async fn declined(
        ctx: Context<'_>,
        inter: &ComponentInteraction,
        user: &User,
    ) -> Result<(), Error> {
        let content = translate!(ctx, "game-declined", user: user.mention().to_string());

        inter
            .create_response(
                ctx,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new()
                        .content(content)
                        .components(CommonButton::accept_or_decline(ctx, true))
                        .allowed_mentions(crate::mentions()),
                ),
            )
            .await?;

        Ok(())
    }

    pub async fn start(
        ctx: Context<'_>,
        inter: &ComponentInteraction,
        contradict: &Contradiction,
    ) -> Result<(), Error> {
        let defender = contradict
            .players
            .iter()
            .find(|p| p.role == Role::Defender)
            .unwrap();
        let content = translate!(ctx, "contradict-start", defender: &defender.name);

        inter
            .create_response(
                ctx,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new()
                        .content(content)
                        .components(Button::display_buttons(ctx, false, true))
                        .allowed_mentions(crate::mentions()),
                ),
            )
            .await?;

        Ok(())
    }

    pub async fn choose(
        ctx: Context<'_>,
        inter: &ComponentInteraction,
        contradict: &Contradiction,
    ) -> Result<Message, Error> {
        let player = contradict.get_player(inter.user.id).unwrap();

        let content = match player.role {
            Role::Defender => translate!(ctx, "choose-shield"),
            _ => translate!(ctx, "choose-weapon"),
        };

        let message = inter
            .create_followup(
                ctx,
                CreateInteractionResponseFollowup::new()
                    .content(content)
                    .components(Button::objects(ctx, inter, contradict)),
            )
            .await?;

        Ok(message)
    }

    pub async fn bet_phase(
        ctx: Context<'_>,
        inter: &ComponentInteraction,
        message_id: MessageId,
    ) -> Result<(), Error> {
        let content = translate!(ctx, "contradict-bet-phase");

        inter
            .edit_followup(
                ctx,
                message_id,
                CreateInteractionResponseFollowup::new()
                    .content(content)
                    .components(Button::display_buttons(ctx, true, false)),
            )
            .await?;

        Ok(())
    }
}

impl Button {
    fn display_buttons(ctx: Context<'_>, choose: bool, bet: bool) -> Vec<CreateActionRow> {
        vec![CreateActionRow::Buttons(vec![
            Self::choose(ctx, choose),
            Self::bet(ctx, bet),
        ])]
    }

    fn choose(ctx: Context<'_>, disabled: bool) -> CreateButton {
        CreateButton::new(format!("{}_choose", ctx.id()))
            .style(ButtonStyle::Secondary)
            .label(translate!(ctx, "choose-object"))
            .disabled(disabled)
    }

    fn bet(ctx: Context<'_>, disabled: bool) -> CreateButton {
        CreateButton::new(format!("{}_bet", ctx.id()))
            .style(ButtonStyle::Secondary)
            .disabled(disabled)
            .label(translate!(ctx, "bet-bios"))
    }

    fn objects(
        ctx: Context<'_>,
        inter: &ComponentInteraction,
        contradict: &Contradiction,
    ) -> Vec<CreateActionRow> {
        let mut buttons = Vec::new();

        let player = contradict.get_player(inter.user.id).unwrap();

        match player.role {
            Role::Defender => {
                for (index, shield) in contradict.shields.iter().enumerate() {
                    let button = CreateButton::new(format!("{}_object_{}", ctx.id(), index))
                        .style(ButtonStyle::Secondary)
                        .label(translate!(ctx, shield.name()))
                        .disabled(shield.used);

                    buttons.push(button);
                }
            }
            _ => {
                for (index, weapon) in contradict.weapons.iter().enumerate() {
                    let button = CreateButton::new(format!("{}_object_{}", ctx.id(), index))
                        .style(ButtonStyle::Secondary)
                        .label(translate!(ctx, weapon.name()))
                        .disabled(weapon.used);

                    buttons.push(button);
                }
            }
        }

        vec![CreateActionRow::Buttons(buttons)]
    }
}
