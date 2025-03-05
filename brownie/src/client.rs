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
        commands::profile::inventory(),
        commands::profile::message(),
        commands::work::work(),
        commands::contradiction::contradiction(),
        commands::nim::nim(),
        commands::dices::dices(),
        commands::system::shop(),
        commands::system::top(),
    ];

    let translations = read_ftl()?;
    apply_translations(&translations, &mut commands);

    let data = Data {
        pool: ::database::connect().await?,
        members: Cache::builder()
            .time_to_live(Duration::from_secs(300))
            .build(),
        translations,
        system: Cache::builder()
            .time_to_live(Duration::from_secs(600))
            .build(),
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
                crate::cache_system(ctx).await;
            })
        },
        post_command: |ctx| {
            Box::pin(async move {
                let data = ctx.data();
                let member = crate::get_member(ctx, ctx.author().id).await.unwrap();
                let mut write = member.write().await;
                write.add_points(15, &data.pool).await.unwrap();
            })
        },
        on_error: |error| Box::pin(on_error(error)),
        allowed_mentions: Some(
            CreateAllowedMentions::new()
                .empty_users()
                .replied_user(false),
        ),
        manual_cooldowns: true,
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
            ctx.reply(error.to_string()).await.unwrap();
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                tracing::error!("error while handling error: {}", e);
            }
        }
    }
}
