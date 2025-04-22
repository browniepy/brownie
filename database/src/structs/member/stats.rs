use super::{get_actual_rpg, Balance, Error, Inventories, Member, PgPool};

impl Member {
    pub fn in_rpg(&self) -> bool {
        self.balance.rpg.is_some()
    }

    pub async fn refresh_profile(&mut self, pool: &PgPool) -> Result<(), Error> {
        let rpg = get_actual_rpg(pool).await?;

        self.balance = Balance::build(pool, self.id, rpg).await?;
        self.inventories = Inventories::build(pool, self.id, rpg).await?;

        Ok(())
    }

    pub async fn join_rpg(&mut self, pool: &PgPool) -> Result<(), Error> {
        if self.in_rpg() {
            return Err("already in rpg".into());
        }

        let current_rpg = super::get_actual_rpg(pool).await?;

        if current_rpg.is_none() {
            return Err("rpg not found".into());
        }

        sqlx::query!(
            "INSERT INTO player (rpg, player)
            VALUES ($1, $2);",
            current_rpg.unwrap(),
            self.id,
        )
        .execute(pool)
        .await?;

        self.refresh_profile(pool).await?;

        Ok(())
    }

    pub async fn leave_rpg(&mut self, pool: &PgPool) -> Result<(), Error> {
        if !self.in_rpg() {
            return Err("not in rpg".into());
        }

        let current_rpg = super::get_actual_rpg(pool).await?;

        if current_rpg.is_none() {
            return Err("rpg not found".into());
        }

        sqlx::query!(
            "DELETE FROM player WHERE rpg = $1 AND player = $2;",
            current_rpg.unwrap(),
            self.id
        )
        .execute(pool)
        .await?;

        self.refresh_profile(pool).await?;

        Ok(())
    }
}
