use crate::{commands::choice::Game, Data};
use database::structs::{club::Club, guild::Guild, Member, System};
use poise::serenity_prelude::{CreateAllowedMentions, GuildId, UserId};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use super::{Context, Error};

pub struct Helper;

impl Helper {
    pub async fn cache_club(ctx: Context<'_>, club_id: i64) {
        let data = ctx.data();

        data.clubs
            .entry_by_ref(&club_id)
            .or_insert_with(async { Club::build(&data.pool, club_id).await.unwrap().into() })
            .await;
    }

    pub async fn get_club(ctx: Context<'_>, club_id: i64) -> Result<Arc<RwLock<Club>>, Error> {
        let data = ctx.data();

        let club_entry = data.clubs.get(&club_id).await;

        match club_entry {
            Some(club) => Ok(club),
            None => {
                Self::cache_club(ctx, club_id).await;

                Ok(data.clubs.get(&club_id).await.unwrap())
            }
        }
    }

    pub async fn invalidate_club(ctx: Context<'_>, club_id: i64) {
        let data = ctx.data();
        data.clubs.invalidate(&club_id).await;
    }
}

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

pub async fn cache_guild(data: &Data, id: GuildId) {
    data.guilds
        .entry_by_ref(id.as_ref())
        .or_insert_with(async {
            tracing::info!("cached guild {}", id);

            Guild::build(&data.pool, id.into()).await.unwrap().into()
        })
        .await;
}

pub async fn cache(ctx: Context<'_>, id: UserId) {
    let data = ctx.data();

    data.members
        .entry_by_ref(id.as_ref())
        .or_insert_with(async {
            let user = id.to_user(ctx).await.unwrap();
            tracing::info!("cached {}", user.name);

            Member::build(&data.pool, id.into()).await.unwrap().into()
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

pub async fn get_guild(data: &Data, id: GuildId) -> Result<Arc<RwLock<Guild>>, Error> {
    let guild_entry = data.guilds.get(id.as_ref()).await;

    match guild_entry {
        Some(guild) => Ok(guild),
        None => {
            cache_guild(data, id).await;

            Ok(data.guilds.get(id.as_ref()).await.unwrap())
        }
    }
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

pub fn points_revenue(bet: i64) -> PointsRevenue {
    let winner = (bet as f32 * 0.1) as i32;
    let loser = (bet as f32 * 0.01) as i32;
    PointsRevenue { winner, loser }
}

pub async fn add_win_points(
    ctx: Context<'_>,
    winner: UserId,
    loser: UserId,
    bet: i64,
) -> Result<(), Error> {
    let data = ctx.data();
    let revenue = points_revenue(bet);

    let winner = get_member(ctx, winner).await?;
    let mut write = winner.write().await;
    write.increase_points(&data.pool, revenue.winner).await?;

    let loser = get_member(ctx, loser).await?;
    let mut write = loser.write().await;
    write.increase_points(&data.pool, revenue.loser).await?;

    Ok(())
}

pub async fn charge_single_bet(
    ctx: Context<'_>,
    id: UserId,
    bet: i64,
    winner: bool,
) -> Result<(), Error> {
    let data = ctx.data();

    let user = get_member(ctx, id).await?;
    let mut write = user.write().await;

    let points_revenue = points_revenue(bet);

    if winner {
        write.increase_bios(&data.pool, bet).await?;
        write
            .increase_points(&data.pool, points_revenue.winner)
            .await?;
    } else {
        write.decrease_bios(&data.pool, bet).await?;
        write
            .increase_points(&data.pool, points_revenue.loser)
            .await?;
    }

    Ok(())
}

pub async fn charge_bet(
    ctx: Context<'_>,
    winner: UserId,
    loser: UserId,
    bet: i64,
    _game: Game,
) -> Result<(), Error> {
    add_win_points(ctx, winner, loser, bet).await?;

    Ok(())
}

pub async fn can_partial_bet(ctx: Context<'_>, id: UserId, bet: i64) -> Result<bool, Error> {
    let player = get_member(ctx, id).await?;
    let player_read = player.read().await;

    let required_bal = (bet as f32 * 1.0) as i64;
    Ok(player_read.get_bios() >= required_bal)
}

pub fn mentions() -> CreateAllowedMentions {
    CreateAllowedMentions::new()
        .everyone(false)
        .replied_user(false)
        .all_users(false)
}

pub async fn set_gamble(ctx: Context<'_>, user_id: UserId) -> Result<(), Error> {
    let member = get_member(ctx, user_id).await?;
    let mut member_write = member.write().await;
    member_write.state.in_gamble = true;

    Ok(())
}

pub async fn free_gamble(ctx: Context<'_>, users_ids: Vec<UserId>) -> Result<(), Error> {
    for user_id in users_ids {
        let member = get_member(ctx, user_id).await?;
        let mut member_write = member.write().await;
        member_write.state.in_gamble = false;
    }

    Ok(())
}
