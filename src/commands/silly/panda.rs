use crate::{config::constants::EMBED_COLOR, Context, Error};
use poise::{serenity_prelude as serenity, CreateReply};
use rand::Rng;
use serde_json::Value;
use serenity::builder::CreateEmbed;

/// Sends a random panda image
#[poise::command(slash_command)]
pub async fn panda(ctx: Context<'_>) -> Result<(), Error> {
    let is_red_panda = rand::rng().random_bool(0.5);
    let animal = if is_red_panda { "red_panda" } else { "panda" };
    let api_url = format!("https://some-random-api.com/img/{}", animal);

    let resp = match reqwest::get(&api_url).await {
        Ok(r) => r,
        Err(_) => {
            ctx.send(
                CreateReply::default()
                    .content("❌ Failed to fetch panda image")
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
                .title(if is_red_panda { "Red Panda!" } else { "Panda!" })
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
