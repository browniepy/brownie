use crate::{helpers::get_member, Context, Error};
use poise::serenity_prelude::UserId;
use types::dices::Selection;

pub struct Parse;

impl Parse {
    pub fn num_with_commas(number: i32) -> String {
        let num_str = number.to_string();
        let len = num_str.len();
        let mut s = String::with_capacity(len + len / 3);

        for (idx, ch) in num_str.chars().enumerate() {
            if idx > 0 && (len - idx) % 3 == 0 {
                s.push(',');
            }
            s.push(ch);
        }
        s
    }

    pub fn format_seconds(seconds: u64) -> String {
        if seconds < 60 {
            format!("{}s", seconds)
        } else {
            let minutes = seconds / 60;
            let remaining_seconds = seconds % 60;

            if remaining_seconds == 0 {
                format!("{}m", minutes)
            } else {
                format!("{}m{}s", minutes, remaining_seconds)
            }
        }
    }

    pub fn abbreviate_number(number: i32) -> String {
        let abs_number = number.abs();

        match abs_number as f64 {
            num if num >= 1_000_000_000.0 => format!("{:.1}b", num / 1_000_000_000.0),
            num if num >= 1_000_000.0 => format!("{:.1}m", num / 1_000_000.0),
            num if num >= 1_000.0 => format!("{:.1}k", num / 1_000.0),
            _ => number.to_string(),
        }
    }

    pub fn abbreviation_to_number(value: &str) -> Result<i32, Error> {
        if let Ok(num) = value.parse::<i32>() {
            return Ok(num);
        }

        let lowercase = value.to_lowercase();

        if lowercase == "all" || lowercase == "half" {
            return Err("Palabra reservada, debe procesarse en la función amount".into());
        }

        let last_char = lowercase.chars().last().unwrap_or_default();

        let multiplier = match last_char {
            'k' => 1_000,
            'm' => 1_000_000,
            'b' => 1_000_000_000,
            _ => {
                return Err(
                    "Formato no válido. Usa un número o abreviaturas como '1k', '2.5m', '1b'"
                        .into(),
                )
            }
        };

        let numeric_part = &lowercase[0..lowercase.len() - 1];

        match numeric_part.parse::<f64>() {
            Ok(num) => Ok((num * multiplier as f64) as i32),
            Err(_) => Err("Formato de número no válido".into()),
        }
    }

    pub async fn amount(
        ctx: Context<'_>,
        user_id: UserId,
        amount: Option<String>,
    ) -> Result<i32, Error> {
        let member = get_member(ctx, user_id).await?;
        let read = member.read().await;

        const MIN_BET: i32 = 500;

        if read.balance < MIN_BET {
            return Err("No tienes suficiente dinero".into());
        }

        let amount_str = match amount {
            Some(amt) => amt,
            None => return Ok(MIN_BET),
        };

        match amount_str.to_lowercase().as_str() {
            "all" => return Ok(read.balance),
            "half" => return Ok(read.balance / 2),
            _ => {}
        }

        let parsed_amount = Self::abbreviation_to_number(&amount_str)?;

        if parsed_amount < MIN_BET {
            return Err(format!("No puedes apostar menos de {}", MIN_BET).into());
        }

        if parsed_amount > read.balance {
            return Err("No tienes suficiente dinero".into());
        }

        Ok(parsed_amount)
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
