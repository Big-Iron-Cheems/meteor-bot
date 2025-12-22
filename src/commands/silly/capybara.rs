use crate::{Ctx, Error, commands::silly::fetch_image_url, config::constants::EMBED_COLOR};
use anyhow::Context;
use poise::{CreateReply, serenity_prelude as serenity};
use serenity::builder::CreateEmbed;
use url::Url;

/// Sends a random capybara image
#[poise::command(slash_command, category = "Silly")]
pub async fn capybara(ctx: Ctx<'_>) -> Result<(), Error> {
    let api_url = Url::parse_with_params("https://api.capy.lol/v1/capybara", [("json", "true")])
        .context("failed to parse capybara API URL")?;
    if let Ok(url) = fetch_image_url(api_url, "data/url").await {
        let embed = CreateEmbed::default().title("Capybara!").color(EMBED_COLOR).image(url);
        ctx.send(CreateReply::default().embed(embed)).await?;
    } else {
        ctx.send(
            CreateReply::default()
                .content("Failed to fetch capybara image")
                .ephemeral(true),
        )
        .await?;
    }

    Ok(())
}
