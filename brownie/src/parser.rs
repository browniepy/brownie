use crate::{helpers::get_member, Context, Error};
use poise::serenity_prelude::UserId;
use types::dices::Selection;

pub struct Parse;

impl Parse {
    pub fn num_with_commas(number: i64) -> String {
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

    pub fn abbreviate_number(number: i64) -> String {
        let abs_number = number.abs();

        match abs_number as f64 {
            num if num >= 1_000_000_000_000_000.0 => {
                format!("{:.1}q", num / 1_000_000_000_000_000.0)
            }
            num if num >= 1_000_000_000_000.0 => format!("{:.1}t", num / 1_000_000_000_000.0),
            num if num >= 1_000_000_000.0 => format!("{:.1}b", num / 1_000_000_000.0),
            num if num >= 1_000_000.0 => format!("{:.1}m", num / 1_000_000.0),
            num if num >= 1_000.0 => format!("{:.1}k", num / 1_000.0),
            _ => number.to_string(),
        }
    }

    pub fn abbreviation_to_number(value: &str) -> Result<i64, Error> {
        // First, check if the input contains a percentage
        if let Some(percentage_result) = Self::parse_percentage(value) {
            return Ok(percentage_result);
        }

        // Existing number parsing logic
        if let Ok(num) = value.parse::<i64>() {
            return Ok(num);
        }

        let lowercase = value.to_lowercase();

        if lowercase == "all" || lowercase == "half" {
            return Err("Palabra reservada, debe procesarse en la función amount".into());
        }

        let last_char = lowercase.chars().last().unwrap_or_default();

        let multiplier: i64 = match last_char {
            'k' => 1_000,
            'm' => 1_000_000,
            'b' => 1_000_000_000,
            't' => 1_000_000_000_000,
            'q' => 1_000_000_000_000_000,
            _ => {
                return Err(
                    "invalid format use k for thousands, m for millions, b for billions, t for trillions, q for quadrillions"
                        .into(),
                )
            }
        };

        let numeric_part = &lowercase[0..lowercase.len() - 1];

        match numeric_part.parse::<f64>() {
            Ok(num) => Ok((num * multiplier as f64) as i64),
            Err(_) => Err("Formato de número no válido".into()),
        }
    }

    // New method to parse percentages
    fn parse_percentage(value: &str) -> Option<i64> {
        let lowercase = value.to_lowercase().replace(' ', "");

        // Check if the value ends with %
        if lowercase.ends_with('%') {
            // Remove the % sign
            let percentage_str = lowercase.trim_end_matches('%');

            // Try to parse percentage without any preceding number
            if let Ok(percentage) = percentage_str.parse::<f64>() {
                return Some(percentage as i64);
            }

            // Check for cases like "7k 30%" or "7000 30%"
            let parts: Vec<&str> = value.split('%').collect();
            if parts.len() == 2 {
                let base_amount_str = parts[0].trim();
                let percentage_str = parts[1].trim();

                // Try to parse base amount using existing method
                if let Ok(base_amount) = Self::abbreviation_to_number(base_amount_str) {
                    if let Ok(percentage) = percentage_str.parse::<f64>() {
                        return Some((base_amount as f64 * (percentage / 100.0)) as i64);
                    }
                }
            }
        }

        None
    }

    pub async fn amount(
        ctx: Context<'_>,
        user_id: UserId,
        amount: Option<String>,
        min_bet: i64,
    ) -> Result<i64, Error> {
        let member = get_member(ctx, user_id).await?;
        let read = member.read().await;

        if read.get_bios() < min_bet {
            return Err("No tienes suficiente dinero".into());
        }

        let amount_str = match amount {
            Some(amt) => amt,
            None => return Ok(min_bet),
        };

        match amount_str.to_lowercase().as_str() {
            "all" => return Ok(read.get_bios()),
            "half" => return Ok(read.get_bios() / 2),
            _ => {}
        }

        let parsed_amount = Self::abbreviation_to_number(&amount_str)?;

        if parsed_amount < min_bet {
            return Err(format!(
                "No puedes apostar menos de {}",
                Self::num_with_commas(min_bet)
            )
            .into());
        }

        if parsed_amount > read.get_bios() {
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
