use crate::{helpers::get_member, Context, Error};
use poise::serenity_prelude::UserId;
use types::dices::Selection;

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

    pub fn dice_choice(choice: String) -> Result<Selection, Error> {
        let result = match choice.to_lowercase().as_str() {
            "pair" => Selection::Pair,
            "unpair" => Selection::Unpair,
            "p" => Selection::Pair,
            "u" => Selection::Unpair,
            _ => {
                return Err("choice must be pair or unpair".into());
            }
        };

        Ok(result)
    }

    pub fn dice_choice_num(choice: String) -> Result<i32, Error> {
        let result = choice.parse::<i32>()?;

        if !(2..=12).contains(&result) {
            return Err("number must be between 2 and 12".into());
        }

        Ok(result)
    }

    pub fn choice_kind(choice: String) -> ChoiceKind {
        let parse_num = choice.parse::<i32>();

        match parse_num {
            Ok(_) => ChoiceKind::Number,
            Err(_) => ChoiceKind::String,
        }
    }
}

pub enum ChoiceKind {
    String,
    Number,
}

impl ChoiceKind {
    pub fn is_num(&self) -> bool {
        matches!(self, ChoiceKind::Number)
    }
}
