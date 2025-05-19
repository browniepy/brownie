use super::{Error, Greeting, PgPool};
use poise::{
    serenity_prelude::{
        CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateMessage, Member, Mentionable,
    },
    CreateReply,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Embed {
    pub thumbnail: Option<String>,
    pub author: Option<String>,
    pub author_icon: Option<String>,
    pub description: Option<String>,
    pub footer: Option<String>,
    pub footer_icon: Option<String>,
    pub color: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Message {
    pub content: Option<String>,
    pub mention: Option<bool>,
    pub embed: Option<Embed>,
}

impl Message {
    pub fn new(content: String) -> Result<Self, Error> {
        let config = format!("#![enable(implicit_some)]\n{}", content);
        Ok(ron::from_str(&config).unwrap())
    }
}

impl Greeting {
    pub fn get_greet_reply(
        &self,
        member: &Member,
        count: i32,
        guild_name: &str,
    ) -> Result<CreateReply, Error> {
        if self.content.is_none() && self.embed.is_none() {
            return Err("first config the greet message".into());
        }

        let replace_placeholders = |template: Option<String>| -> Option<String> {
            template.as_ref().map(|text| {
                text.replace("-name", member.user.name.as_str())
                    .replace("-mention", member.mention().to_string().as_str())
                    .replace("-guild", guild_name)
                    .replace("-count", count.to_string().as_str())
            })
        };

        let mut builder = CreateReply::default();

        if let Some(final_content) = replace_placeholders(self.content.clone()) {
            builder = builder.content(final_content);
        }

        if let Some(ref embed_template) = self.embed {
            let mut embed = CreateEmbed::default();

            if let Some(ref thumbnail) = embed_template.thumbnail {
                embed = embed.thumbnail(thumbnail.clone());
            }

            if let Some(final_author_name) = replace_placeholders(embed_template.author.clone()) {
                let mut author = CreateEmbedAuthor::new(final_author_name);

                if let Some(ref url) = embed_template.author_icon_url {
                    author = author.icon_url(url.clone());
                }

                embed = embed.author(author);
            }

            if let Some(final_description) =
                replace_placeholders(embed_template.description.clone())
            {
                embed = embed.description(final_description);
            }

            if let Some(final_footer) = replace_placeholders(embed_template.footer.clone()) {
                let mut footer = CreateEmbedFooter::new(final_footer);

                if let Some(ref url) = embed_template.footer_icon_url {
                    footer = footer.icon_url(url.clone());
                }

                embed = embed.footer(footer);
            }

            if let Some(ref color) = embed_template.colour {
                let clean_hex = color.strip_prefix('#').unwrap_or(color);

                embed = embed.color(u32::from_str_radix(clean_hex, 16).unwrap_or(0));
            }

            if let Some(ref image_url) = embed_template.image_url {
                embed = embed.image(image_url.clone());
            }

            builder = builder.embed(embed);
        }

        Ok(builder)
    }

    pub fn get_greet_message(
        &self,
        member: &Member,
        count: i32,
        guild_name: &str,
    ) -> Option<CreateMessage> {
        if !self.enabled {
            return None;
        }

        if self.content.is_none() && self.embed.is_none() {
            return None;
        }

        let replace_placeholders = |template: Option<String>| -> Option<String> {
            template.as_ref().map(|text| {
                text.replace("-name", member.user.name.as_str())
                    .replace("-mention", member.mention().to_string().as_str())
                    .replace("-guild", guild_name)
                    .replace("-count", count.to_string().as_str())
            })
        };

        let mut builder = CreateMessage::default();

        if let Some(final_content) = replace_placeholders(self.content.clone()) {
            builder = builder.content(final_content);
        }

        if let Some(ref embed_template) = self.embed {
            let mut embed = CreateEmbed::default();

            if let Some(ref thumbnail) = embed_template.thumbnail {
                embed = embed.thumbnail(thumbnail.clone());
            }

            if let Some(final_author_name) = replace_placeholders(embed_template.author.clone()) {
                let mut author = CreateEmbedAuthor::new(final_author_name);

                if let Some(ref url) = embed_template.author_icon_url {
                    author = author.icon_url(url.clone());
                }

                embed = embed.author(author);
            }

            if let Some(final_description) =
                replace_placeholders(embed_template.description.clone())
            {
                embed = embed.description(final_description);
            }

            if let Some(final_footer) = replace_placeholders(embed_template.footer.clone()) {
                let mut footer = CreateEmbedFooter::new(final_footer);

                if let Some(ref url) = embed_template.footer_icon_url {
                    footer = footer.icon_url(url.clone());
                }

                embed = embed.footer(footer);
            }

            if let Some(ref color) = embed_template.colour {
                let clean_hex = color.strip_prefix('#').unwrap_or(color);

                embed = embed.color(u32::from_str_radix(clean_hex, 16).unwrap_or(0));
            }

            builder = builder.embed(embed);
        }

        Some(builder)
    }

    pub async fn update_message(&mut self, pool: &PgPool, config: String) -> Result<(), Error> {
        let message = Message::new(config)?;

        sqlx::query!(
            "UPDATE greeting SET content = $1,
            mention = $2
            WHERE id = $3;",
            message.content,
            message.mention.unwrap_or(false),
            self.id
        )
        .execute(pool)
        .await?;

        self.content = message.content;
        self.mention = message.mention.unwrap_or(false);

        if let Some(embed) = message.embed {
            sqlx::query!(
                "INSERT INTO greet_embed (greeting, thumbnail_image_url,
                author, author_icon_url,
                description,
                footer, footer_icon_url, color,
                image_url)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                ON CONFLICT (greeting) DO UPDATE
                SET thumbnail_image_url = $2,
                author = $3,
                author_icon_url = $4,
                description = $5,
                footer = $6,
                footer_icon_url = $7,
                color = $8,
                image_url = $9;",
                self.id,
                embed.thumbnail,
                embed.author,
                embed.author_icon,
                embed.description,
                embed.footer,
                embed.footer_icon,
                embed.color,
                embed.image_url
            )
            .execute(pool)
            .await?;

            self.embed.as_mut().map(|greet_embed| {
                greet_embed.thumbnail = embed.thumbnail;
                greet_embed.author = embed.author;
                greet_embed.author_icon_url = embed.author_icon;
                greet_embed.description = embed.description;
                greet_embed.footer = embed.footer;
                greet_embed.footer_icon_url = embed.footer_icon;
                greet_embed.colour = embed.color;
                greet_embed.image_url = embed.image_url;
            });
        } else if message.embed.is_none() {
            sqlx::query!("DELETE FROM greet_embed WHERE greeting = $1;", self.id)
                .execute(pool)
                .await?;

            self.embed = None;
        }

        Ok(())
    }
}
