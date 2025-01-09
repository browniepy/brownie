use crate::{serenity::*, translation::translate, Context, Error};
use database::models::Role;
use poise::CreateReply;

pub async fn work(ctx: Context<'_>, amount: i32) -> Result<(), Error> {
    ctx.send(CreateReply::default().content(translate!(ctx, "kariume-work", amount: amount)))
        .await?;

    Ok(())
}

pub async fn profile(
    ctx: Context<'_>,
    name: &str,
    roles: Vec<Role>,
    range: Option<i32>,
) -> Result<(), Error> {
    let tr_roles: Vec<String> = roles
        .iter()
        .map(|role| match role {
            Role::Member => translate!(ctx, "member-role"),
            Role::Referee => translate!(ctx, "referee-role", range: range.unwrap()),
            Role::Leader => translate!(ctx, "leader-role"),
            Role::Baku => String::from("Baku"),
            Role::Slave => translate!(ctx, "slave-role"),
            Role::User => translate!(ctx, "user-role"),
        })
        .collect();

    let roles_str = tr_roles.join(", ");

    ctx.send(
        CreateReply::default().content(translate!(ctx, "profile", name: name, role: roles_str)),
    )
    .await?;

    Ok(())
}
