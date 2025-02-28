use database::structs::{Member, System};
use poise::serenity_prelude::UserId;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use super::{Context, Error};

pub async fn cache_system(ctx: Context<'_>) {
    let data = ctx.data();

    data.system
        .entry_by_ref(&())
        .or_insert_with(async {
            tracing::info!("cached system");

            System::new(&data.pool).await.into()
        })
        .await;
}

pub async fn get_system(ctx: Context<'_>) -> Arc<Mutex<System>> {
    let data = ctx.data();

    let system_entry = data.system.get(&()).await;

    match system_entry {
        Some(system) => system,
        None => {
            cache_system(ctx).await;

            data.system.get(&()).await.unwrap()
        }
    }
}

pub async fn cache(ctx: Context<'_>, id: UserId) {
    let data = ctx.data();

    data.members
        .entry_by_ref(id.as_ref())
        .or_insert_with(async {
            let user = id.to_user(ctx).await.unwrap();
            tracing::info!("cached {}", user.name);

            Member::builder(id.into())
                .build(&data.pool)
                .await
                .unwrap()
                .into()
        })
        .await;
}

pub async fn refresh_cache(ctx: Context<'_>, id: UserId) {
    let data = ctx.data();

    let entry = data.members.get(id.as_ref()).await;

    if entry.is_some() {
        data.members.remove(id.as_ref()).await;
    }

    cache(ctx, id).await;
}

pub async fn get_member(ctx: Context<'_>, id: UserId) -> Result<Arc<RwLock<Member>>, Error> {
    let data = ctx.data();

    let member_entry = data.members.get(id.as_ref()).await;

    match member_entry {
        Some(member) => Ok(member),
        None => {
            cache(ctx, id).await;

            Ok(data.members.get(id.as_ref()).await.unwrap())
        }
    }
}

pub struct PointsRevenue {
    pub winner: i32,
    pub loser: i32,
}

pub fn points_revenue(bet: i32) -> PointsRevenue {
    let winner = (bet as f32 * 0.1) as i32;
    let loser = (bet as f32 * 0.01) as i32;
    PointsRevenue { winner, loser }
}

pub async fn add_win_points(
    ctx: Context<'_>,
    winner: UserId,
    loser: UserId,
    bet: i32,
) -> Result<(), Error> {
    let data = ctx.data();
    let revenue = points_revenue(bet);

    let winner = get_member(ctx, winner).await?;
    let mut write = winner.write().await;
    write.add_points(revenue.winner, &data.pool).await?;

    let loser = get_member(ctx, loser).await?;
    let mut write = loser.write().await;
    write.add_points(revenue.loser, &data.pool).await?;

    Ok(())
}

pub async fn charge_single_bet(
    ctx: Context<'_>,
    id: UserId,
    bet: i32,
    winner: bool,
) -> Result<(), Error> {
    let data = ctx.data();

    let user = get_member(ctx, id).await?;
    let mut write = user.write().await;

    let points_revenue = points_revenue(bet);

    if winner {
        write.add_balalance(bet, &data.pool).await?;
        write.add_points(points_revenue.winner, &data.pool).await?;
    } else {
        write.remove_balance(bet, &data.pool).await?;
        write.add_points(points_revenue.loser, &data.pool).await?;
    }

    Ok(())
}

pub async fn charge_bet(
    ctx: Context<'_>,
    winner: UserId,
    loser: UserId,
    bet: i32,
) -> Result<(), Error> {
    let data = ctx.data();

    let loser_player = get_member(ctx, loser).await?;
    let mut loser_write = loser_player.write().await;

    if loser_write.balance >= bet {
        loser_write.remove_balance(bet, &data.pool).await?;

        let winner = get_member(ctx, winner).await?;
        let mut winner_write = winner.write().await;

        winner_write.add_balalance(bet, &data.pool).await?;
    } else {
        let debt = bet - loser_write.balance;
        let revenue = bet - debt;

        loser_write.remove_balance(revenue, &data.pool).await?;
        loser_write
            .set_debt(winner.into(), debt, &data.pool)
            .await?;

        let winner = get_member(ctx, winner).await?;
        let mut winner_write = winner.write().await;

        winner_write.add_balalance(revenue, &data.pool).await?;
    }

    add_win_points(ctx, winner, loser, bet).await?;

    Ok(())
}

pub async fn can_partial_bet(ctx: Context<'_>, id: UserId, bet: i32) -> Result<bool, Error> {
    let player = get_member(ctx, id).await?;
    let player_read = player.read().await;

    let required_bal = (bet as f32 * 1.0) as i32;
    Ok(player_read.balance >= required_bal)
}
