use poise::serenity_prelude::{ComponentInteraction, MessageId, UserId};
use tokio::sync::mpsc::Sender;
use types::{airpoker::AirPoker, blackjack::Blackjack};

pub struct AirData<S> {
    pub sender: Sender<S>,
    pub airpoker: AirPoker,
    pub ok_selected: Vec<UserId>,
    pub ok_bet: Vec<UserId>,
    pub last_inter: Option<ComponentInteraction>,
    pub first_round: bool,
    pub message: MessageId,
}
