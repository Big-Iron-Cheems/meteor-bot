use crate::{config::constants::EMBED_COLOR, Context, Error};
use poise::{serenity_prelude as serenity, CreateReply};
use serde_json::Value;
use serenity::builder::CreateEmbed;

/// Sends a random capybara image
#[poise::command(slash_command)]
pub async fn capybara(ctx: Context<'_>) -> Result<(), Error> {
    let api_url = "https://api.capy.lol/v1/capybara?json=true";

    let resp = match reqwest::get(api_url).await {
        Ok(r) => r,
        Err(_) => {
            ctx.send(
                CreateReply::default()
                    .content("❌ Failed to fetch capybara image")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    let json = match resp.json::<Value>().await {
        Ok(j) => j,
        Err(_) => {
            ctx.send(
                CreateReply::default()
                    .content("❌ Failed to decode the response")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    let url = json
        .get("data")
        .and_then(|d| d.get("url"))
        .and_then(|u| u.as_str());

    match url {
        Some(url) => {
            let embed = CreateEmbed::default()
                .title("Capybara!")
                .color(EMBED_COLOR)
                .image(url);

            ctx.send(CreateReply::default().embed(embed)).await?;
        }
        None => {
            ctx.send(
                CreateReply::default()
                    .content("❌ Failed to parse the response")
                    .ephemeral(true),
            )
            .await?;
        }
    }

    Ok(())
}
