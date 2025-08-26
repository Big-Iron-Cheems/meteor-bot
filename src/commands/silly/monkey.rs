use crate::{constants::EMBED_COLOR, Context, Error};
use poise::{serenity_prelude as serenity, CreateReply};
use rand::Rng;
use serenity::builder::CreateEmbed;

/// Sends a random monkey image
#[poise::command(slash_command)]
pub async fn monkey(ctx: Context<'_>) -> Result<(), Error> {
    let (w, h): (u32, u32) = {
        let mut rng = rand::rng();
        (rng.random_range(200..=1000), rng.random_range(200..=1000))
    };

    let url = format!("https://www.placemonkeys.com/{}/{}?random", w, h);

    let embed = CreateEmbed::default()
        .title("Monkey!")
        .color(EMBED_COLOR)
        .image(&url);

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}
