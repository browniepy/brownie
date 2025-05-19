use chrono::{Datelike, Weekday};
use database::structs::Item;

use crate::{get_member, translate, Context, Error, Parser};

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn daily(ctx: Context<'_>) -> Result<(), Error> {
    let member = get_member(ctx, ctx.author().id).await?;
    let mut member = member.write().await;
    let data = ctx.data();

    if member.state.can_claim_daily {
        let weekday_reward = weekday_reward(ctx).await;
        let mut rewards = Vec::new();

        let bios_amount = Parser::num_with_commas(weekday_reward.bios);
        let bios = translate!(ctx, "daily-bios", amount: bios_amount);
        rewards.push(bios);
        member
            .increase_bios(&data.pool, weekday_reward.bios)
            .await?;

        let points_amount = Parser::num_with_commas(weekday_reward.points as i64);
        let points = translate!(ctx, "daily-points", amount: points_amount);
        rewards.push(points);
        member
            .increase_points(&data.pool, weekday_reward.points)
            .await?;

        if let Some(item) = weekday_reward.item {
            rewards.push(item.name);
            member
                .add_item(&data.pool, item.model_item, item.amount)
                .await?;
        }

        // registra que el usuario ya reclamó este día
        member.log_claim_daily(&data.pool).await?;

        let content = translate!(ctx, "daily-claimed");
        ctx.reply(format!("{}\n{}", content, rewards.join(", ")))
            .await?;
        return Ok(());
    }

    let content = translate!(ctx, "daily-already-claimed");
    ctx.reply(content).await?;
    Ok(())
}

pub struct WeekdayItem {
    pub id: i32,
    pub name: String,
    pub amount: i32,
    pub model_item: Item,
}

impl WeekdayItem {
    pub async fn new(ctx: Context<'_>, id: i32, amount: i32) -> Self {
        let data = ctx.data();
        let system = data.system.get(&()).await.unwrap();
        let lock = system.lock().await;

        let item = lock.get_item_by_id(&data.pool, id).await.unwrap();
        let name = translate!(ctx, &item.name);

        Self {
            id,
            name,
            amount,
            model_item: item,
        }
    }
}

pub struct Daily {
    pub bios: i64,
    pub points: i32,
    pub item: Option<WeekdayItem>,
}

impl Daily {
    pub fn new(bios: i64, points: i32, item: Option<WeekdayItem>) -> Self {
        Self { bios, points, item }
    }
}

pub async fn weekday_reward(ctx: Context<'_>) -> Daily {
    let weekday = chrono::Utc::now().weekday();

    match weekday {
        Weekday::Mon => {
            // coffee
            let item = WeekdayItem::new(ctx, 8, 1).await;
            Daily::new(4500, 2500, Some(item))
        }
        Weekday::Tue => {
            // kariume
            let item = WeekdayItem::new(ctx, 4, 1).await;
            Daily::new(2500, 1500, Some(item))
        }
        Weekday::Wed => Daily::new(2500, 1500, None),
        Weekday::Thu => Daily::new(2500, 1500, None),
        Weekday::Fri => {
            // life insurance
            let item = WeekdayItem::new(ctx, 9, 1).await;
            Daily::new(4500, 1500, Some(item))
        }
        Weekday::Sat => Daily::new(2500, 2000, None),
        Weekday::Sun => Daily::new(2500, 2000, None),
    }
}
