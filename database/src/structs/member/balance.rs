use sqlx::PgPool;

use super::{Error, Member};

impl Member {
    pub fn get_bios(&self) -> i64 {
        self.balance.bios
    }

    pub async fn increase_bios<'a, E>(&mut self, executor: E, bios: i64) -> Result<(), Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        sqlx::query!(
            "UPDATE member SET balance = balance + $1 WHERE id = $2;",
            bios,
            self.id
        )
        .execute(executor)
        .await?;

        self.balance.bios += bios;
        Ok(())
    }

    pub async fn decrease_bios<'a, E>(&mut self, executor: E, bios: i64) -> Result<(), Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        sqlx::query!(
            "UPDATE member SET balance = balance - $1 WHERE id = $2;",
            bios,
            self.id
        )
        .execute(executor)
        .await?;

        self.balance.bios -= bios;
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
