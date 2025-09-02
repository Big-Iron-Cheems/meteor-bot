use crate::{commands::silly::fetch_image_url, config::constants::EMBED_COLOR, Ctx, Error};
use poise::{serenity_prelude as serenity, CreateReply};
use serenity::builder::CreateEmbed;

/// Sends a random capybara image
#[poise::command(slash_command, category = "Silly")]
pub async fn capybara(ctx: Ctx<'_>) -> Result<(), Error> {
    if let Ok(url) = fetch_image_url("https://api.capy.lol/v1/capybara?json=true", "data/url").await {
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
