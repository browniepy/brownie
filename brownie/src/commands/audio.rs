use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};

use crate::{Context, Error};

#[derive(Serialize, Deserialize, Debug)]
struct Attachment {
    id: u32,
    filename: String,
    description: Option<String>,
    waveform: String,
    duration_secs: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct PayloadJson {
    tts: Option<bool>,
    flags: u32,
    attachments: Vec<Attachment>,
}

#[poise::command(
    prefix_command,
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn audio(ctx: Context<'_>, media: String) -> Result<(), Error> {
    ctx.defer().await?;

    let bytes = reqwest::get(media).await?.bytes().await?;

    let channel = ctx.guild_channel().await.unwrap();

    let payload_json = PayloadJson {
        tts: None,
        flags: 8192,
        attachments: vec![Attachment {
            id: 0,
            filename: "audio.wav".to_string(),
            description: None,
            waveform: "AA==".to_string(),
            duration_secs: 10.0,
        }],
    };
    let payload_json_string = serde_json::to_string(&payload_json)?;

    use reqwest::multipart;

    let form = multipart::Form::new()
        .text("payload_json", payload_json_string)
        .part(
            "files[0]",
            multipart::Part::bytes(bytes.to_vec()).file_name("audio.wav".to_string()),
        );

    let client = reqwest::Client::new();

    let url = format!(
        "https://discord.com/api/v10/channels/{}/messages",
        channel.id
    );
    client
        .post(url)
        .header(
            AUTHORIZATION,
            format!("Bot {}", std::env::var("token").unwrap()),
        )
        .multipart(form)
        .send()
        .await?;

    Ok(())
}
