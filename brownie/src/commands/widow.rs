use crate::{
    types::widow::{Player, Widow},
    Context, Error,
};
use poise::serenity_prelude::{ComponentInteraction, ComponentInteractionCollector, User};
use tokio::sync::mpsc;

enum Signal {
    NextRound(ComponentInteraction),
    FinalRound(ComponentInteraction),
    GameEnd(ComponentInteraction),
}

enum Event {
    Interaction(ComponentInteraction),
    Receiver(Signal),
    Timeout,
}

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "economy"
)]
pub async fn widow(ctx: Context<'_>, user: User) -> Result<(), Error> {
    let mut widow = Widow::new(Player::new(ctx.author().clone()), Player::new(user.clone()));

    let (tx, mut rx) = mpsc::channel::<Signal>(5);

    loop {
        let timeout = widow
            .round_timeout
            .map(|duration| tokio::time::sleep(duration));
        let timeout = async {
            if let Some(sleep) = timeout {
                sleep.await;
                true
            } else {
                std::future::pending::<bool>().await
            }
        };

        let collector = ComponentInteractionCollector::new(ctx);

        let event = tokio::select! {
            _ = timeout => Event::Timeout,
            inter = collector.next() => Event::Interaction(inter.unwrap()),
            signal = rx.recv() => Event::Receiver(signal.unwrap()),
        };

        match event {
            Event::Receiver(signal) => {}
            Event::Interaction(inter) => {
                if inter.data.custom_id == format!("{}_decline", ctx.id()) {
                    break;
                }

                if inter.data.custom_id == format!("{}_accept", ctx.id()) {}

                if inter.data.custom_id == format!("{}_pass", ctx.id()) {}

                if inter.data.custom_id == format!("{}_tock", ctx.id()) {}
            }
            Event::Timeout => {}
        }
    }
    Ok(())
}
