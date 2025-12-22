use crate::{Ctx, Error, config::constants::EMBED_COLOR};
use anyhow::Context;
use poise::{CreateReply, serenity_prelude as serenity};
use rand::Rng;
use serenity::CreateEmbed;
use url::Url;

/// Sends a random monkey image
#[poise::command(slash_command, category = "Silly")]
pub async fn monkey(ctx: Ctx<'_>) -> Result<(), Error> {
    let (w, h): (u32, u32) = {
        let mut rng = rand::rng();
        (rng.random_range(200..=1000), rng.random_range(200..=1000))
    };

    let mut api_url = Url::parse("https://www.placemonkeys.com/")
        .and_then(|base| base.join(&format!("{w}/{h}")))
        .context("failed to build placemonkeys URL")?;
    api_url.query_pairs_mut().append_pair("random", "");

    let embed = CreateEmbed::default()
        .title("Monkey!")
        .color(EMBED_COLOR)
        .image(api_url);

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}
