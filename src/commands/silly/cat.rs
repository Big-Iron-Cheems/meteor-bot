use crate::{config::constants::EMBED_COLOR, Ctx, Error};
use poise::{serenity_prelude as serenity, CreateReply};
use serde_json::Value;
use serenity::builder::CreateEmbed;

/// Sends a random cat image
#[poise::command(slash_command, category = "Silly")]
pub async fn cat(ctx: Ctx<'_>) -> Result<(), Error> {
    let api_url = "https://some-random-api.com/img/cat";

    let resp = match reqwest::get(api_url).await {
        Ok(r) => r,
        Err(_) => {
            ctx.send(
                CreateReply::default()
                    .content("❌ Failed to fetch cat image")
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

    let url = json.get("link").and_then(|u| u.as_str());

    match url {
        Some(url) => {
            let embed = CreateEmbed::default()
                .title("Cat!")
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
