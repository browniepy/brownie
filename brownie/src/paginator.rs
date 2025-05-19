use crate::{serenity, translate, Context};
use poise::CreateReply;

pub struct PageField {
    pub title: String,
    pub description: String,
}

pub async fn paginate(ctx: Context<'_>, pages: Vec<PageField>) -> Result<(), serenity::Error> {
    let ctx_id = ctx.id();
    let mut current_page = 0;
    let pages_per_embed = 3;
    let total_pages = pages.len().div_ceil(pages_per_embed);

    let build_components = |current_page: usize| {
        let mut prev = serenity::CreateButton::new(format!("{}_prev", ctx_id))
            .label(translate!(ctx, "prev"))
            .style(serenity::ButtonStyle::Secondary);

        let mut next = serenity::CreateButton::new(format!("{}_next", ctx_id))
            .label(translate!(ctx, "next"))
            .style(serenity::ButtonStyle::Secondary);

        if current_page == 0 {
            prev = prev.disabled(true);
        }
        if current_page + 1 >= total_pages {
            next = next.disabled(true);
        }

        serenity::CreateActionRow::Buttons(vec![prev, next])
    };

    let build_embed = |current_page: usize| {
        let mut embed = serenity::CreateEmbed::default();
        let start = current_page * pages_per_embed;
        let end = (start + pages_per_embed).min(pages.len());

        for page in &pages[start..end] {
            embed = embed.field(&page.title, &page.description, false);
        }

        embed
    };

    let reply = CreateReply::default()
        .embed(build_embed(current_page))
        .components(vec![build_components(current_page)])
        .allowed_mentions(crate::mentions());

    ctx.send(reply).await?;

    while let Some(press) = serenity::collector::ComponentInteractionCollector::new(ctx)
        .timeout(std::time::Duration::from_secs(300))
        .await
    {
        if press.user.id != ctx.author().id {
            continue;
        }

        let custom_id = &press.data.custom_id;

        if custom_id == &format!("{}_next", ctx_id) && current_page + 1 < total_pages {
            current_page += 1;
        } else if custom_id == &format!("{}_prev", ctx_id) && current_page > 0 {
            current_page -= 1;
        } else {
            continue;
        }

        press
            .create_response(
                ctx.serenity_context(),
                serenity::CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::new()
                        .embed(build_embed(current_page))
                        .components(vec![build_components(current_page)]),
                ),
            )
            .await?;
    }

    Ok(())
}
