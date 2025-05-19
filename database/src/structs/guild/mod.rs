use crate::{Error, PgPool};

mod greet;
use greet::Greeting;

mod messages;

pub struct Guild {
    pub id: i64,
    pub greeting: Greeting,
}

impl Guild {
    pub async fn build(pool: &PgPool, id: i64) -> Result<Self, Error> {
        let greeting = Greeting::build(pool, id).await?;
        Ok(Self { id, greeting })
    }

    pub async fn toggle_greeting(&mut self, pool: &PgPool) -> Result<bool, Error> {
        sqlx::query!(
            "UPDATE greeting SET enabled = NOT enabled WHERE id = $1;",
            self.greeting.id
        )
        .execute(pool)
        .await?;

        self.greeting.enabled = !self.greeting.enabled;

        Ok(self.greeting.enabled)
    }
}
