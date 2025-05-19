use crate::{translate, Context, Error, Parser};
use database::error::WorkError;
use std::time::Duration;

#[poise::command(
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "economy",
    subcommand_required,
    subcommands("apply", "leave", "list", "shift")
)]
pub async fn work(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "economy"
)]
pub async fn shift(ctx: Context<'_>) -> Result<(), Error> {
    let member = crate::get_member(ctx, ctx.author().id).await?;
    let mut write = member.write().await;

    {
        let mut cooldown_tracker = ctx.command().cooldowns.lock().unwrap();

        let cooldown_durations = poise::CooldownConfig {
            user: Some(Duration::from_secs(write.get_work_cooldown() as u64)),
            ..Default::default()
        };

        match cooldown_tracker.remaining_cooldown(ctx.cooldown_context(), &cooldown_durations) {
            Some(remaining) => {
                let time = Parser::format_seconds(remaining.as_secs());
                let content = translate!(ctx, "work-cooldown", time: time);
                return Err(content.into());
            }
            None => cooldown_tracker.start_cooldown(ctx.cooldown_context()),
        }
    };

    let data = ctx.data();
    let earned = write.work(&data.pool).await?;

    let content = match write.job.as_ref() {
        None => translate!(ctx, "unknown-work", amount: earned),
        Some(job) => match job.name.as_str() {
            "referee" => translate!(ctx, "referee-work", amount: earned),
            "kariume" => translate!(ctx, "kariume-work", amount: earned),
            _ => translate!(ctx, "unknown-work", amount: earned),
        },
    };

    ctx.reply(content).await?;

    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "economy"
)]
pub async fn apply(ctx: Context<'_>, id: i32) -> Result<(), Error> {
    {
        let system = crate::get_system(ctx).await;
        let lock = system.lock().await;

        if !lock.jobs.iter().any(|job| job.id == id) {
            return Err(translate!(ctx, "work-not-found").into());
        }
    }

    let member = crate::get_member(ctx, ctx.author().id).await?;
    let mut write = member.write().await;

    if let Err(error) = write.apply_job(&ctx.data().pool, id).await {
        let content = match error {
            WorkError::AlreadyEmployed => translate!(ctx, "work-apply-employed"),
            WorkError::CannotApply => translate!(ctx, "work-apply-cannot"),
            _ => translate!(ctx, "unknown-error"),
        };

        return Err(content.into());
    }

    let content = translate!(ctx, "work-apply-message");
    ctx.reply(content).await?;

    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "economy"
)]
pub async fn leave(ctx: Context<'_>) -> Result<(), Error> {
    let member = crate::get_member(ctx, ctx.author().id).await?;
    let mut write = member.write().await;

    if let Err(error) = write.leave_job(&ctx.data().pool).await {
        let content = match error {
            WorkError::NotEmployed => translate!(ctx, "work-leave-unemployed"),
            _ => translate!(ctx, "unknown-error"),
        };

        return Err(content.into());
    };

    let content = translate!(ctx, "work-leave-message");
    ctx.reply(content).await?;

    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "economy"
)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    let system = crate::get_system(ctx).await;
    let lock = system.lock().await;

    let jobs = lock.jobs.iter().map(|job| {
        let cooldown = Parser::format_seconds(job.cooldown as u64);
        crate::PageField {
            title: translate!(ctx, &job.name),
            description: translate!(ctx, "job-info", level: job.required_points, salary: job.salary[0], cooldown: cooldown),
        }
    }).collect::<Vec<_>>();

    crate::paginate(ctx, jobs).await?;

    Ok(())
}
