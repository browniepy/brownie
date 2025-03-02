use poise::serenity_prelude::{
    CreateInteractionResponse, CreateInteractionResponseMessage, MessageId, User,
};

use super::{responses, ComponentInteraction, Context, Contradiction, Error, Role};

pub async fn handle_inter(
    ctx: Context<'_>,
    inter: ComponentInteraction,
    contradict: &mut Contradiction,
    bet: i32,
    user: &User,
    message: MessageId,
) -> Result<(), Error> {
    if inter.data.custom_id == format!("{}_accept", ctx.id()) && inter.user.id == user.clone().id {
        if crate::can_partial_bet(ctx, inter.user.id, bet).await? {
            contradict.init_roles();
            responses::start(ctx, &inter, contradict).await?;
        } else {
            inter
                .create_response(
                    ctx,
                    CreateInteractionResponse::UpdateMessage(
                        CreateInteractionResponseMessage::new()
                            .content("no tienes por lo menos el 80% de dinero para aceptar")
                            .components(vec![]),
                    ),
                )
                .await?;

            return Err("no dinero".into());
        }
    }

    if inter.data.custom_id == format!("{}_decline", ctx.id()) && inter.user.id == user.clone().id {
        responses::declined(ctx, &inter).await?;
        return Err("rechazado".into());
    }

    if contradict
        .players
        .iter()
        .any(|player| player.id == inter.user.id)
    {
        if inter.data.custom_id == format!("{}_choose", ctx.id()) {
            inter.defer_ephemeral(ctx).await?;

            let msg = responses::choose_object(ctx, &inter, contradict).await?;

            let player = contradict.get_mut_player(inter.user.id).unwrap();
            player.set_ephemeral(msg);
        }

        if inter.data.custom_id == format!("{}_bet", ctx.id()) {
            if contradict.already_bet.contains(&inter.user.id) {
                inter
                    .create_response(ctx, CreateInteractionResponse::Acknowledge)
                    .await?;
            } else {
                let player = contradict.get_player(inter.user.id).unwrap();
                responses::bet_modal(ctx, &inter, player).await?;
            }
        }

        if let Some(index) = inter
            .data
            .custom_id
            .strip_prefix(&format!("{}_object_", ctx.id()))
        {
            if let Ok(index) = index.parse::<usize>() {
                inter.defer(ctx).await?;

                let player = contradict.get_mut_player(inter.user.id).unwrap();

                inter
                    .delete_followup(ctx, &player.ephemeral.clone().unwrap().id)
                    .await?;

                player.delete_ephemeral();

                match player.role {
                    Role::Defender => {
                        contradict.select_shield(index);
                    }
                    Role::Attacker => {
                        contradict.select_weapon(index);
                    }
                    Role::None => {}
                }

                if contradict.all_selected() {
                    responses::bet_phase(ctx, &inter, message).await?;
                }
            }
        }
    }

    Ok(())
}
