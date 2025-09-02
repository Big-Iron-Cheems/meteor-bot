use crate::{commands::silly::fetch_image_url, config::constants::EMBED_COLOR, Ctx, Error};
use poise::{serenity_prelude as serenity, CreateReply};
use serenity::builder::CreateEmbed;

/// Sends a random dog image
#[poise::command(slash_command, category = "Silly")]
pub async fn dog(ctx: Ctx<'_>) -> Result<(), Error> {
    if let Ok(url) = fetch_image_url("https://some-random-api.com/img/dog", "link").await {
        let embed = CreateEmbed::default().title("Dog!").color(EMBED_COLOR).image(url);
        ctx.send(CreateReply::default().embed(embed)).await?;
    } else {
        ctx.send(
            CreateReply::default()
                .content("Failed to fetch dog image")
                .ephemeral(true),
        )
        .await?;
    }

    Ok(())
}
