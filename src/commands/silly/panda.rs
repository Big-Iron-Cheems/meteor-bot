use crate::{commands::silly::fetch_image_url, config::constants::EMBED_COLOR, Ctx, Error};
use poise::{serenity_prelude as serenity, CreateReply};
use rand::Rng;
use serenity::builder::CreateEmbed;

/// Sends a random panda image
#[poise::command(slash_command, category = "Silly")]
pub async fn panda(ctx: Ctx<'_>) -> Result<(), Error> {
    let (animal, title) = if rand::rng().random_bool(0.5) {
        ("red_panda", "Red Panda!")
    } else {
        ("panda", "Panda!")
    };
    let api_url = format!("https://some-random-api.com/img/{}", animal);

    if let Ok(url) = fetch_image_url(&api_url, "link").await {
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
