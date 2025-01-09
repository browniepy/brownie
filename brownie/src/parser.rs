use crate::{get_member, Context, Error};
use poise::serenity_prelude::UserId;

pub struct Parse;

impl Parse {
    pub async fn amount(
        ctx: Context<'_>,
        user_id: UserId,
        amount: Option<String>,
    ) -> Result<i32, Error> {
        let member = get_member(ctx, user_id).await?;
        let read = member.read().await;

        Ok(match amount {
            Some(amount) => match amount.parse::<i32>() {
                Ok(val) => {
                    if val < 500 {
                        return Err("No puedes apostar menos de 500".into());
                    }
                    val
                }
                Err(_) => match amount.to_lowercase().as_str() {
                    "all" => {
                        if read.balance < 500 {
                            return Err("No tienes suficiente dinero".into());
                        }

                        read.balance
                    }
                    "half" => {
                        if read.balance < 500 {
                            return Err("No tienes suficiente dinero".into());
                        }

                        read.balance / 2
                    }
                    _ => return Err("Error".into()),
                },
            },
            None => {
                if read.balance > 500 {
                    500
                } else {
                    return Err("No tienes suficiente dinero".into());
                }
            }
        })
    }
}
