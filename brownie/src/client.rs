use crate::{
    commands,
    helpers::get_guild,
    translation::{apply_translations, read_ftl},
    Cache, Data, Duration, Error,
};
use poise::serenity_prelude::{ChannelId, Client, ClientBuilder, FullEvent, GatewayIntents};

pub async fn build() -> Result<Client, Error> {
    let token = std::env::var("token").unwrap();
    let intents = GatewayIntents::all();

    let mut commands = vec![
        commands::work::work(),
        commands::contradiction::contradict(),
        commands::nim::nim(),
        commands::rewards::daily(),
        commands::profile::balance(),
        commands::profile::points(),
        commands::profile::inventory(),
        commands::give::give(),
        commands::greeting::greet(),
        commands::club::club(),
    ];

    let translations = read_ftl()?;
    apply_translations(&translations, &mut commands);

    let data = Data {
        pool: ::database::connect().await?,
        members: Cache::builder()
            .time_to_live(Duration::from_secs(600))
            .build(),
        guilds: Cache::builder()
            .time_to_live(Duration::from_secs(600))
            .build(),
        translations,
        system: Cache::builder()
            .time_to_live(Duration::from_secs(600))
            .build(),
        clubs: Cache::builder()
            .time_to_live(Duration::from_secs(600))
            .build(),
    };

    let options = poise::FrameworkOptions {
        commands,
        pre_command: |ctx| {
            Box::pin(async move {
                ctx.defer().await.unwrap();

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
                write.increase_points(&data.pool, 15).await.unwrap();

                if let Some(category) = &ctx.command().category {
                    if category == "gambling" {
                        write.state.in_gamble = false;
                    }
                }
            })
        },
        on_error: |error| Box::pin(on_error(error)),
        event_handler: |ctx, event, framework, data| {
            Box::pin(event_handler(ctx, event, framework, data))
        },
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
        poise::FrameworkError::CommandCheckFailed { error, ctx, .. } => {
            let error = error.unwrap_or("None".into());
            ctx.reply(error.to_string()).await.unwrap();
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                tracing::error!("error while handling error: {}", e);
            }
        }
    }
}

async fn event_handler(
    ctx: &poise::serenity_prelude::Context,
    event: &poise::serenity_prelude::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        FullEvent::GuildMemberAddition { new_member } => {
            let guild = get_guild(data, new_member.guild_id).await?;
            let read = guild.read().await;

            if let Some(id) = read.greeting.channel {
                let channel = ctx.http.get_channel(ChannelId::new(id as u64)).await?;
                let channel = channel.guild().unwrap();

                if let Some(builder) = read.greeting.get_greet_message(new_member, 0, "hola") {
                    channel.send_message(ctx, builder).await?;
                }
            }
        }
        _ => {}
    }

    Ok(())
}
