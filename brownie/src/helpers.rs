use database::structs::{Member, System};
use poise::serenity_prelude::UserId;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{Context, Error};

pub async fn cache_system(ctx: Context<'_>) {
    let data = ctx.data();

    data.system
        .entry_by_ref(&())
        .or_insert_with(async {
            tracing::info!("cached system");

            System::new(&data.pool).await
        })
        .await;
}

pub async fn get_system(ctx: Context<'_>) -> Arc<System> {
    let data = ctx.data();

    let system_entry = data.system.get(&()).await;

    match system_entry {
        Some(system) => system.into(),
        None => {
            cache_system(ctx).await;

            data.system.get(&()).await.unwrap().into()
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

pub async fn charge_bet(
    ctx: Context<'_>,
    winner: UserId,
    loser: UserId,
    bet: i32,
) -> Result<(), Error> {
    let data = ctx.data();

    let loser = get_member(ctx, loser).await?;
    let mut loser_write = loser.write().await;

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

    Ok(())
}

pub async fn can_partial_bet(ctx: Context<'_>, id: UserId, bet: i32) -> Result<bool, Error> {
    let player = get_member(ctx, id).await?;
    let player_read = player.read().await;

    let required_bal = (bet as f32 * 0.8) as i32;
    Ok(player_read.balance >= required_bal)
}
