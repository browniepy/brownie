use super::{Error, Member};
use crate::models::{JobModel, Role};
use rand::Rng;
use sqlx::PgPool;

impl Member {
    pub fn get_work_cooldown(&self) -> i32 {
        self.job.clone().unwrap_or_default().cooldown
    }

    pub fn is_employed(&self) -> bool {
        self.job.is_some()
    }

    pub async fn work(&mut self, pool: &PgPool) -> Result<i64, Error> {
        let range: i64 = match &self.job {
            Some(job) => {
                let range = &job.salary;
                rand::thread_rng().gen_range(range[0].into()..range[1].into())
            }
            None => rand::thread_rng().gen_range(500..900),
        };

        self.increase_yn(pool, range).await?;
        Ok(range)
    }

    pub async fn leave_job(&mut self, pool: &PgPool) -> Result<(), Error> {
        if !self.is_employed() {
            return Err("not employed".into());
        }

        sqlx::query!("UPDATE member SET job = NULL WHERE id = $1", self.id)
            .execute(pool)
            .await?;

        self.job = None;

        Ok(())
    }

    pub async fn apply_job(&mut self, pool: &PgPool, id: i32) -> Result<(), Error> {
        if self.is_employed() {
            return Err("already employed".into());
        }

        let record = sqlx::query!("SELECT can_apply_job($1, $2);", self.id, id)
            .fetch_one(pool)
            .await?;

        if !record.can_apply_job.unwrap() {
            return Err("cant apply to job".into());
        }

        sqlx::query!("UPDATE member SET job = $1 WHERE id = $2;", id, self.id)
            .execute(pool)
            .await?;

        let job = sqlx::query_as!(
            JobModel,
            "SELECT job.id, job.name,
            job.salary AS \"salary: Vec<i32>\",
            job.required_role AS \"required_role: Role\",
            job.required_points, job.cooldown
            FROM job INNER JOIN member ON job.id = member.job
            WHERE member.id = $1;",
            self.id
        )
        .fetch_optional(pool)
        .await?;

        self.job = job;

        Ok(())
    }
}
