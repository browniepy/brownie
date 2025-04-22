use sqlx::PgPool;

use super::{Error, Member};

impl Member {
    pub fn get_yn(&self) -> i64 {
        self.balance.normal.yn
    }

    pub async fn increase_yn<'a, E>(&mut self, executor: E, yn: i64) -> Result<(), Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        sqlx::query!(
            "UPDATE member SET balance = balance + $1 WHERE id = $2;",
            yn,
            self.id
        )
        .execute(executor)
        .await?;

        self.balance.normal.yn += yn;
        Ok(())
    }

    pub async fn decrease_yn<'a, E>(&mut self, executor: E, yn: i64) -> Result<(), Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        sqlx::query!(
            "UPDATE member SET balance = balance - $1 WHERE id = $2;",
            yn,
            self.id
        )
        .execute(executor)
        .await?;

        self.balance.normal.yn -= yn;
        Ok(())
    }

    pub fn get_bios(&self) -> i64 {
        self.balance.rpg.clone().unwrap().bios
    }

    pub async fn increase_bios<'a, E>(
        &mut self,
        executor: E,
        bios: i64,
        rpg_id: Option<i32>,
    ) -> Result<(), Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        if !self.in_rpg() {
            return Err("not in rpg".into());
        }

        if rpg_id.is_none() {
            return Err("rpg not found".into());
        }

        sqlx::query!(
            "UPDATE player SET balance = balance + $1 WHERE player = $2 AND rpg = $3;",
            bios,
            self.id,
            rpg_id.unwrap()
        )
        .execute(executor)
        .await?;

        if let Some(ref mut rpg) = self.balance.rpg {
            rpg.bios += bios;
        } else {
            return Err("No se ha encontrado el rpg".into());
        }

        Ok(())
    }

    pub async fn decrease_bios<'a, E>(
        &mut self,
        executor: E,
        bios: i64,
        rpg_id: Option<i32>,
    ) -> Result<(), Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        if !self.in_rpg() {
            return Err("not in rpg".into());
        }

        if rpg_id.is_none() {
            return Err("rpg not found".into());
        }

        sqlx::query!(
            "UPDATE player SET balance = balance - $1 WHERE player = $2 AND rpg = $3;",
            bios,
            self.id,
            rpg_id.unwrap()
        )
        .execute(executor)
        .await?;

        if let Some(ref mut rpg) = self.balance.rpg {
            rpg.bios -= bios;
        } else {
            return Err("No se ha encontrado el rpg".into());
        }

        Ok(())
    }

    pub async fn log_claim_daily(&mut self, pool: &PgPool) -> Result<(), Error> {
        if !self.state.can_claim_daily {
            return Err("already claimed".into());
        }

        sqlx::query!("SELECT register_daily_reward_claim($1);", self.id)
            .fetch_one(pool)
            .await?;

        self.state.can_claim_daily = false;
        Ok(())
    }
}
