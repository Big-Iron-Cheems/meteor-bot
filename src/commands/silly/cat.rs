use crate::{Ctx, Error, commands::silly::fetch_image_url, config::constants::EMBED_COLOR};
use anyhow::Context;
use poise::{CreateReply, serenity_prelude as serenity};
use serenity::builder::CreateEmbed;
use url::Url;

/// Sends a random cat image
#[poise::command(slash_command, category = "Silly")]
pub async fn cat(ctx: Ctx<'_>) -> Result<(), Error> {
    let api_url = Url::parse("https://some-random-api.com/img/cat").context("failed to parse cat API URL")?;
    if let Ok(url) = fetch_image_url(api_url, "link").await {
        let embed = CreateEmbed::default().title("Cat!").color(EMBED_COLOR).image(url);
        ctx.send(CreateReply::default().embed(embed)).await?;
    } else {
        ctx.send(
            CreateReply::default()
                .content("Failed to fetch cat image")
                .ephemeral(true),
        )
        .await?;
    }

    Ok(())
}
