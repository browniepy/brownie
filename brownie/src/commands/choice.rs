#[derive(poise::ChoiceParameter)]
pub enum StlRule {
    Karamete,
    RequiredBios,
}

#[derive(poise::ChoiceParameter)]
pub enum CoinType {
    Yens,
    Bios,
}

#[derive(poise::ChoiceParameter)]
pub enum Game {
    Contradict,
    NimTypeZero,
    BlackJack,
    RussianRoulette,
    Dices,
    Falaris,
    ECard,
    AirPoker,
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Game::Contradict => write!(f, "Contradict"),
            Game::NimTypeZero => write!(f, "NimTypeZero"),
            Game::BlackJack => write!(f, "BlackJack"),
            Game::RussianRoulette => write!(f, "Rr"),
            Game::Dices => write!(f, "Dices"),
            Game::Falaris => write!(f, "Falaris"),
            Game::ECard => write!(f, "ECard"),
            Game::AirPoker => write!(f, "AirPoker"),
        }
    }
}
