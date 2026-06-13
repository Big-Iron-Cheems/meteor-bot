use crate::{Ctx, Error, commands::silly::fetch_image_url, config::constants::EMBED_COLOR};
use anyhow::Context;
use poise::{CreateReply, serenity_prelude as serenity};
use rand::RngExt;
use serenity::CreateEmbed;
use url::Url;

/// Sends a random panda image
#[poise::command(slash_command, category = "Silly")]
pub async fn panda(ctx: Ctx<'_>) -> Result<(), Error> {
    let (animal, title) = if rand::rng().random_bool(0.5) {
        ("red_panda", "Red Panda!")
    } else {
        ("panda", "Panda!")
    };

    let api_url = Url::parse("https://some-random-api.com/img/")
        .and_then(|base| base.join(animal))
        .context("failed to build panda API URL")?;
    if let Ok(url) = fetch_image_url(&ctx.data().http_client, api_url, "/link").await {
        let embed = CreateEmbed::default().title(title).color(EMBED_COLOR).image(url);
        ctx.send(CreateReply::default().embed(embed)).await?;
    } else {
        ctx.send(
            CreateReply::default()
                .content("Failed to fetch panda image")
                .ephemeral(true),
        )
        .await?;
    }

    Ok(())
}
