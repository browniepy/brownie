#[derive(Clone, Debug)]
pub enum Value {
    Zero,
    One,
    Two,
    Three,
}

#[derive(Clone, Debug)]
pub struct Card {
    pub value: Value,
    pub disabled: bool,
}

impl Card {
    pub fn new(value: Value) -> Self {
        Self {
            value,
            disabled: false,
        }
    }

    pub fn standart_deck() -> Vec<Self> {
        use rand::seq::SliceRandom;

        let mut deck = Vec::new();

        for _ in 0..10 {
            deck.extend([
                Self::new(Value::Zero),
                Self::new(Value::One),
                Self::new(Value::Two),
                Self::new(Value::Three),
            ]);
        }

        deck.shuffle(&mut rand::thread_rng());
        deck
    }

    pub fn value(&self) -> i32 {
        match self.value {
            Value::Zero => 0,
            Value::One => 1,
            Value::Two => 2,
            Value::Three => 3,
        }
    }

    pub fn name(&self) -> String {
        match self.value {
            Value::Zero => String::from("zero"),
            Value::One => String::from("one"),
            Value::Two => String::from("two"),
            Value::Three => String::from("three"),
        }
    }
}
