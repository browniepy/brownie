use rand::Rng;
use sqlx::PgPool;
use types::cards::poker::Card;

use crate::{
    models::{Debt, ItemInventory, JobModel, MemberModel, Role, StatModel},
    structs::Gamble,
    ErrorT,
};

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Member {
    pub id: i64,
    pub balance: i32,
    pub inventory: HashMap<i32, ItemInventory>,
    pub level: i32,
    pub points: i32,
    pub roles: Vec<Role>,
    pub referee_range: Option<i32>,
    pub personal_referee_id: Option<i64>,
    pub profile_text: Option<String>,
    pub job: Option<JobModel>,
    pub stats: Vec<StatModel>,
    pub deck: Vec<Card>,
    pub gamble: Gamble,
    pub debt: Vec<Debt>,
}

impl Member {
    pub fn builder(id: i64) -> Self {
        Self {
            id,
            balance: 0,
            inventory: HashMap::new(),
            level: 0,
            points: 0,
            roles: Vec::new(),
            referee_range: None,
            personal_referee_id: None,
            profile_text: None,
            job: None,
            stats: Vec::new(),
            deck: Vec::new(),
            gamble: Gamble::None,
            debt: Vec::new(),
        }
    }

    pub async fn build(self, pool: &PgPool) -> Result<Self, ErrorT> {
        sqlx::query!(
            "INSERT INTO member (id) VALUES ($1) ON CONFLICT (id) DO NOTHING",
            self.id
        )
        .execute(pool)
        .await?;

        let stats = sqlx::query_as!(
            MemberModel,
            "SELECT
            balance,
            points,
            roles AS \"roles: Vec<Role> \",
            referee_range,
            personal_referee,
            profile_text
            FROM member WHERE id = $1",
            self.id
        )
        .fetch_one(pool)
        .await?;

        let items = sqlx::query_as!(
            ItemInventory,
            "SELECT item.id, item.name, item.description, inventory.amount FROM inventory
            INNER JOIN item ON inventory.item = item.id WHERE inventory.member = $1;",
            self.id
        )
        .fetch_all(pool)
        .await?;

        let items = items
            .iter()
            .map(|item| (item.id.unwrap(), item.to_owned()))
            .collect::<HashMap<i32, ItemInventory>>();

        // get job where member id is $1, and return the job name, salary range, description, required level and required role
        let job = sqlx::query_as!(
            JobModel,
            "SELECT job.name,
            job.description,
            job.salary_range AS \"salary_range: Vec<i32>\",
            job.required_role AS \"required_role: Role\",
            job.required_level,
            job.cooldown
            FROM job
            INNER JOIN member ON job.name = member.job WHERE member.id = $1",
            self.id
        )
        .fetch_optional(pool)
        .await?;

        let debt = sqlx::query_as!(
            Debt,
            "SELECT to_member AS to, amount FROM debs WHERE member = $1;",
            self.id
        )
        .fetch_all(pool)
        .await
        .unwrap();

        let statistics = sqlx::query_as!(
            StatModel,
            "SELECT game, victories, defeats, victory_text, defeat_text FROM statistics WHERE member = $1",
            self.id
        ).fetch_all(pool).await?;

        Ok(Self {
            id: self.id,
            balance: stats.balance,
            inventory: items,
            level: self.calculate_level(),
            points: stats.points,
            roles: stats.roles.unwrap_or(Vec::new()),
            personal_referee_id: stats.personal_referee,
            referee_range: stats.referee_range,
            profile_text: stats.profile_text,
            job,
            stats: statistics,
            deck: Card::standart_deck(),
            gamble: self.gamble,
            debt,
        })
    }

    pub fn calculate_level(&self) -> i32 {
        if self.points < 1100 {
            return 1;
        }

        use std::f64;
        (12.0 * f64::ln(self.points as f64 / 1000.0) + 1.0).floor() as i32
    }

    pub fn get_work_cooldown(&self) -> i32 {
        self.job.clone().unwrap_or_default().cooldown
    }

    pub async fn get_mut_statistics(&mut self, pool: &PgPool, game: &str) -> &mut StatModel {
        // get or create a new StatModel
        if self.stats.iter_mut().any(|x| x.game == game) {
            self.stats.iter_mut().find(|x| x.game == game).unwrap()
        } else {
            sqlx::query!(
                "INSERT INTO statistics (member, game) VALUES ($1, $2)",
                self.id,
                game
            )
            .execute(pool)
            .await
            .unwrap();

            let stat = sqlx::query_as!(
                StatModel,
                "SELECT game, victories, defeats, victory_text, defeat_text FROM statistics WHERE member = $1 AND game = $2",
                self.id,
                game
            ).fetch_one(pool).await.unwrap();

            self.stats.push(stat);
            self.stats.last_mut().unwrap()
        }
    }

    // return a vec with id of the members that the user has debt with
    pub fn get_debt_users(&self) -> Vec<i64> {
        self.debt.iter().map(|x| x.to.unwrap()).collect()
    }

    pub async fn get_debt(&self, id: i64) -> Option<&Debt> {
        self.debt.iter().find(|x| x.to == Some(id))
    }

    pub async fn set_debt(&mut self, id: i64, amount: i32, pool: &PgPool) -> Result<(), ErrorT> {
        let debt = self.debt.iter_mut().find(|x| x.to == Some(id));

        if let Some(debt) = debt {
            sqlx::query!(
                "UPDATE debs SET amount = amount + $1 WHERE member = $2 AND to_member = $3",
                amount,
                self.id,
                id
            )
            .execute(pool)
            .await
            .unwrap();

            debt.amount = Some(debt.amount.unwrap() + amount);
        } else {
            sqlx::query!(
                "INSERT INTO debs (member, to_member, amount) VALUES ($1, $2, $3)",
                self.id,
                id,
                amount
            )
            .execute(pool)
            .await
            .unwrap();

            self.debt.push(Debt {
                to: Some(id),
                amount: Some(amount),
            });
        }

        Ok(())
    }

    pub async fn get_statistics(&self, game: &str) -> Option<&StatModel> {
        if let Some(stat) = self.stats.iter().find(|x| x.game == game) {
            Some(stat)
        } else {
            None
        }
    }

    pub fn set_gamble(&mut self, gamble: Gamble) {
        self.gamble = gamble
    }

    pub fn reset_gamble(&mut self) {
        self.gamble = Gamble::None
    }

    pub fn in_gamble(&self) -> bool {
        self.gamble == Gamble::None
    }

    pub fn reload_deck(&mut self) {
        if self.deck.len() < 10 {
            self.deck = Card::standart_deck();
        }
    }

    // get statistic info by name
    pub fn get_stat(&self, game: &str) -> Option<&StatModel> {
        self.stats.iter().find(|x| x.game == game)
    }

    pub async fn remove_balance(&mut self, amount: i32, pool: &PgPool) -> Result<(), ErrorT> {
        sqlx::query!(
            "UPDATE member SET balance = balance - $1 WHERE id = $2",
            amount,
            self.id
        )
        .execute(pool)
        .await?;
        self.balance -= amount;
        Ok(())
    }

    pub async fn get_victory_text(&self, game: &str, pool: &PgPool) -> Option<String> {
        let stat = self.get_stat(game);
        if let Some(stat) = stat {
            return stat.victory_text.clone();
        }

        let stat = sqlx::query!(
            "SELECT victory_text FROM statistics WHERE member = $1 AND game = $2",
            self.id,
            game
        )
        .fetch_one(pool)
        .await
        .ok()?;

        stat.victory_text
    }

    pub async fn change_victory_text(
        &mut self,
        game: String,
        text: Option<String>,
        pool: &PgPool,
    ) -> Result<(), ErrorT> {
        sqlx::query!(
            "UPDATE statistics
            SET victory_text = $1 WHERE game = $2 AND member = $3",
            text,
            game,
            self.id
        )
        .execute(pool)
        .await?;
        self.get_mut_statistics(pool, &game).await.victory_text = text;
        Ok(())
    }

    pub async fn change_defeat_text(
        &mut self,
        game: String,
        text: Option<String>,
        pool: &PgPool,
    ) -> Result<(), ErrorT> {
        sqlx::query!(
            "UPDATE statistics
            SET defeat_text = $1 WHERE game = $2 AND member = $3",
            text,
            game,
            self.id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn add_victory(&mut self, game: String, pool: &PgPool) -> Result<(), ErrorT> {
        sqlx::query!(
            "UPDATE statistics
            SET victories = victories + 1 WHERE game = $1 AND member = $2",
            game,
            self.id
        )
        .execute(pool)
        .await?;
        self.get_mut_statistics(pool, &game).await.victories += 1;
        Ok(())
    }

    pub async fn add_defeat(&mut self, game: String, pool: &PgPool) -> Result<(), ErrorT> {
        sqlx::query!(
            "UPDATE statistics
            SET defeats = defeats + 1 WHERE game = $1 AND member = $2",
            game,
            self.id
        )
        .execute(pool)
        .await?;
        self.get_mut_statistics(pool, &game).await.defeats += 1;
        Ok(())
    }

    pub async fn change_profile_text(
        &mut self,
        text: Option<String>,
        pool: &PgPool,
    ) -> Result<(), ErrorT> {
        sqlx::query!(
            "UPDATE member SET profile_text = $1 WHERE id = $2",
            text,
            self.id
        )
        .execute(pool)
        .await?;
        self.profile_text = text;
        Ok(())
    }

    pub async fn can_stl(&self, pool: &PgPool) -> Result<bool, ErrorT> {
        if !self.roles.contains(&Role::Member) {
            return Ok(false);
        }

        if self.balance < 50_000_000 {
            return Ok(false);
        }

        if let Some(referee_id) = self.personal_referee_id {
            let referee =
                sqlx::query!("SELECT referee_range FROM member WHERE id = $1", referee_id)
                    .fetch_one(pool)
                    .await?;

            if referee.referee_range.unwrap() == 0 {
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    pub async fn assign_referee(&mut self, referee: i64, pool: &PgPool) -> Result<(), ErrorT> {
        let res = sqlx::query!(
            "SELECT personal_referee FROM member WHERE personal_referee = $1;",
            referee
        )
        .fetch_one(pool)
        .await?;

        if res.personal_referee.is_some() {
            return Err("No se puede asignar a este Referee".into());
        }

        sqlx::query!(
            "UPDATE member SET personal_referee = $1 WHERE id = $2",
            referee,
            self.id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn give_role(&mut self, role: Role, pool: &PgPool) -> Result<(), ErrorT> {
        if self.roles.contains(&role) {
            return Err("This user already has that role".into());
        }

        if role == Role::Referee {
            sqlx::query!("SELECT assign_referee_range($1);", self.id)
                .execute(pool)
                .await?;

            let range = sqlx::query!(
                "SELECT referee_range FROM member
                 WHERE id = $1",
                self.id
            )
            .fetch_one(pool)
            .await?;

            self.referee_range = range.referee_range;
            self.roles.push(role);

            return Ok(());
        }

        sqlx::query!(
            "UPDATE member SET roles = array_append(roles, $1) WHERE id = $2",
            role as _,
            self.id
        )
        .execute(pool)
        .await?;

        self.roles.push(role);

        Ok(())
    }

    pub async fn add_points(&mut self, amount: i32, pool: &PgPool) -> Result<(), ErrorT> {
        sqlx::query!(
            "UPDATE member SET points = points + $1 WHERE id = $2",
            amount,
            self.id
        )
        .execute(pool)
        .await?;

        self.points += amount;

        Ok(())
    }

    pub async fn add_balalance(&mut self, amount: i32, pool: &PgPool) -> Result<(), ErrorT> {
        sqlx::query!(
            "UPDATE member SET balance = balance + $1 WHERE id = $2",
            amount,
            self.id
        )
        .execute(pool)
        .await?;
        self.balance += amount;
        Ok(())
    }

    pub async fn work(&mut self, pool: &PgPool) -> Result<i32, ErrorT> {
        let num = if self.job.is_none() {
            rand::thread_rng().gen_range(200..800)
        } else {
            let job = self.job.as_ref().unwrap();
            let range = job.salary_range.as_ref().unwrap();
            rand::thread_rng().gen_range(range[0]..range[1])
        };

        self.add_balalance(num, pool).await?;
        Ok(num)
    }
}
