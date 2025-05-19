use super::{Error, Member};
use sqlx::PgPool;

impl Member {
    pub fn get_points(&self) -> i32 {
        self.balance.points
    }

    pub async fn increase_points(&mut self, pool: &PgPool, points: i32) -> Result<(), Error> {
        sqlx::query!(
            "UPDATE member SET points = points + $1 WHERE id = $2;",
            points,
            self.id
        )
        .execute(pool)
        .await?;

        self.balance.points += points;
        Ok(())
    }
}
