use super::{Error, PgPool};

pub struct GreetEmbed {
    pub id: i32,
    pub thumbnail: Option<String>,
    pub author: Option<String>,
    pub author_icon_url: Option<String>,
    pub description: Option<String>,
    pub footer: Option<String>,
    pub footer_icon_url: Option<String>,
    pub colour: Option<String>,
    pub image_url: Option<String>,
}

pub struct Greeting {
    pub id: i32,
    pub guild: i64,
    pub channel: Option<i64>,
    pub enabled: bool,
    pub content: Option<String>,
    pub mention: bool,
    pub embed: Option<GreetEmbed>,
}

impl Greeting {
    pub async fn build(pool: &PgPool, id: i64) -> Result<Self, Error> {
        sqlx::query!(
            "INSERT INTO greeting (guild)
            VALUES ($1)
            ON CONFLICT DO NOTHING;",
            id
        )
        .execute(pool)
        .await?;

        let record = sqlx::query!(
            "SELECT id, channel, enabled, content, mention
            FROM greeting
            WHERE guild = $1;",
            id
        )
        .fetch_one(pool)
        .await?;

        let embed_record = sqlx::query!(
            "SELECT id, thumbnail_image_url, author, author_icon_url, description, footer, footer_icon_url, color, image_url
            FROM greet_embed
            WHERE greeting = $1;",
            record.id
        )
        .fetch_optional(pool)
        .await?;

        Ok(Self {
            id: record.id,
            channel: record.channel,
            enabled: record.enabled,
            content: record.content,
            mention: record.mention,
            guild: id,
            embed: match embed_record {
                Some(record) => Some(GreetEmbed {
                    id: record.id,
                    thumbnail: record.thumbnail_image_url,
                    author: record.author,
                    author_icon_url: record.author_icon_url,
                    description: record.description,
                    footer: record.footer,
                    footer_icon_url: record.footer_icon_url,
                    colour: record.color,
                    image_url: record.image_url,
                }),
                None => None,
            },
        })
    }

    pub async fn set_channel(&mut self, pool: &PgPool, channel: Option<i64>) -> Result<(), Error> {
        sqlx::query!(
            "UPDATE greeting SET channel = $1 WHERE id = $2;",
            channel,
            self.id
        )
        .execute(pool)
        .await?;

        self.channel = channel;

        Ok(())
    }
}
