use super::{Error, Member};
use sqlx::PgPool;

impl Member {
    pub fn get_points(&self) -> i32 {
        self.balance.normal.points
    }

    pub async fn increase_points(&mut self, pool: &PgPool, points: i32) -> Result<(), Error> {
        sqlx::query!(
            "UPDATE member SET points = points + $1 WHERE id = $2;",
            points,
            self.id
        )
        .execute(pool)
        .await?;

        self.balance.normal.points += points;
        Ok(())
    }

    pub fn get_experience(&self) -> Result<i32, Error> {
        self.balance
            .rpg
            .as_ref()
            .ok_or("not in rpg".into())
            .map(|rpg| rpg.exp)
    }

    pub fn get_level(&self) -> Result<i32, Error> {
        self.balance
            .rpg
            .as_ref()
            .ok_or("not in rpg".into())
            .map(|rpg| rpg.level)
    }

    pub async fn increase_experience(&mut self, pool: &PgPool, amount: i32) -> Result<(), Error> {
        let balance = self.balance.rpg.as_mut().ok_or("not in rpg")?;
        let actual_rpg = super::get_actual_rpg(pool).await?;

        if actual_rpg.is_none() {
            return Err("rpg not found".into());
        }

        sqlx::query!(
            "UPDATE player SET experience = experience + $1
            WHERE rpg = $2 AND player = $3;",
            amount,
            actual_rpg.unwrap(),
            self.id
        )
        .execute(pool)
        .await?;

        balance.exp += amount;
        Ok(())
    }

    pub async fn increase_level(&mut self, pool: &PgPool, amount: i32) -> Result<(), Error> {
        let balance = self.balance.rpg.as_mut().ok_or("not in rpg")?;
        let actual_rpg = super::get_actual_rpg(pool).await?;

        if actual_rpg.is_none() {
            return Err("rpg not found".into());
        }

        sqlx::query!(
            "UPDATE player SET level = level + $1
            WHERE rpg = $2 AND player = $3;",
            amount,
            actual_rpg.unwrap(),
            self.id
        )
        .execute(pool)
        .await?;

        balance.level += amount;
        Ok(())
    }
}
