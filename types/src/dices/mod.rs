mod player;
pub use player::Player;

#[derive(poise::ChoiceParameter, Clone, Hash, Eq, PartialEq, Debug)]
pub enum Selection {
    Pair,
    Unpair,
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub enum DiceResult {
    NumWin,
    ChoiceWin,
    NumberLose,
    ChoiceLose,
    Lose,
}

pub struct Dices {
    pub player: Player,
    pub dices: Vec<i32>,
}

impl Dices {
    pub fn new(player: Player) -> Self {
        Self {
            player,
            dices: Vec::new(),
        }
    }

    pub fn roll_dices(&mut self) {
        self.dices = vec![self.roll_dice(), self.roll_dice()];
    }

    pub fn player_result(&self) -> DiceResult {
        let number = self.dices.iter().sum::<i32>();

        if let Some(choice) = &self.player.selection {
            if *choice == self.check_pair(number) {
                return DiceResult::ChoiceWin;
            }

            return DiceResult::ChoiceLose;
        }

        if let Some(num) = self.player.number {
            if num == number {
                return DiceResult::NumWin;
            }

            return DiceResult::NumberLose;
        }

        DiceResult::Lose
    }

    pub fn check_pair(&self, number: i32) -> Selection {
        match number % 2 == 0 {
            true => Selection::Pair,
            false => Selection::Unpair,
        }
    }

    fn roll_dice(&self) -> i32 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen_range(1..=6)
    }
}
