use poise::serenity_prelude::CreateAllowedMentions;

use crate::{
    commands,
    serenity::{Client, ClientBuilder, GatewayIntents},
    translation::{apply_translations, read_ftl},
    Cache, Data, Duration, Error,
};

pub async fn build() -> Result<Client, Error> {
    let token = std::env::var("token").unwrap();
    let intents = GatewayIntents::all();

    let mut commands = vec![
        commands::profile::balance(),
        commands::profile::stl(),
        commands::work::work(),
        commands::dth::dth(),
        commands::zeronim::nim(),
        commands::audio::audio(),
        commands::oldmaid::oldmaid(),
        commands::airpoker::airpoker(),
        commands::blackjack::blackjack(),
    ];

    let translations = read_ftl()?;
    apply_translations(&translations, &mut commands);

    let data = Data {
        pool: ::database::connect().await?,
        members: Cache::builder()
            .time_to_live(Duration::from_secs(300))
            .build(),
        translations,
    };

    let options = poise::FrameworkOptions {
        commands,
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("!".into()),
            ..Default::default()
        },
        pre_command: |ctx| {
            Box::pin(async move {
                let command = &ctx.command().qualified_name;
                let author = &ctx.author().name;

                tracing::info!("{} used {}", author, command);
                crate::cache(ctx, ctx.author().id).await;

                let member = crate::get_member(ctx, ctx.author().id).await.unwrap();
                let mut write = member.write().await;

                write.reload_deck();
            })
        },
        on_error: |error| Box::pin(on_error(error)),
        allowed_mentions: Some(
            CreateAllowedMentions::new()
                .empty_users()
                .replied_user(false),
        ),
        ..Default::default()
    };

    let framework = poise::Framework::builder()
        .setup(move |ctx, _, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(data)
            })
        })
        .options(options)
        .build();

    Ok(ClientBuilder::new(token.as_str(), intents)
        .framework(framework)
        .await?)
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,);
            ctx.reply(format!("{:?}", error)).await.unwrap();
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}
