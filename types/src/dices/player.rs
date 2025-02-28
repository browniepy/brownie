use super::Selection;
use inflector::Inflector;
use poise::serenity_prelude::{User, UserId};

#[derive(Clone, Debug)]
pub struct Player {
    pub id: UserId,
    pub name: String,
    pub bet: i32,
    pub selection: Option<Selection>,
    pub number: Option<i32>,
}

impl Player {
    pub fn new(user: &User, bet: i32) -> Self {
        let name = user.display_name().to_title_case();

        Self {
            id: user.id,
            name,
            bet,
            selection: None,
            number: None,
        }
    }

    pub fn choice(&mut self, choice: Selection) {
        self.selection = Some(choice);
    }

    pub fn number(&mut self, number: i32) {
        self.number = Some(number);
    }

    pub fn has_selection(&self) -> bool {
        self.selection.is_some() || self.number.is_some()
    }
}
