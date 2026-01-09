use crate::{Ctx, Error, config::constants::EMBED_COLOR};
use anyhow::Context;
use poise::{CreateReply, serenity_prelude as serenity};
use serenity::CreateEmbed;
use tracing::error;

/// Shows various stats about Meteor
#[poise::command(slash_command, category = "Utility")]
pub async fn stats(
    ctx: Ctx<'_>,
    #[description = "The date to fetch the stats for (DD-MM-YYYY)"] date: Option<String>,
) -> Result<(), Error> {
    let date = match validate_date(date) {
        Ok(d) => d,
        Err(e) => {
            ctx.send(CreateReply::default().content(e.to_string()).ephemeral(true))
                .await?;
            return Ok(());
        }
    };

    let stats = match fetch_stats(ctx, &date).await {
        Ok(stats) => stats,
        Err(e) => {
            error!("Error fetching stats for {date}: {e}");
            ctx.send(
                CreateReply::default()
                    .content("Failed to fetch stats for this date.")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    let gained = stats.joins - stats.leaves;
    let content = format!(
        "**Date**: {}\n**Joins**: {}\n**Leaves**: {}\n**Gained**: {}\n**Downloads**: {}",
        stats.date, stats.joins, stats.leaves, gained, stats.downloads
    );

    let embed = CreateEmbed::default()
        .title("Meteor Stats")
        .description(content)
        .color(EMBED_COLOR);

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

fn validate_date(date: Option<String>) -> Result<String, Error> {
    date.map_or_else(
        || Ok(chrono::Local::now().format("%d-%m-%Y").to_string()),
        |d| {
            chrono::NaiveDate::parse_from_str(&d, "%d-%m-%Y")
                .map(|_| d)
                .context("Invalid date format. Please use DD-MM-YYYY.")
        },
    )
}

#[derive(serde::Deserialize)]
struct StatsResponse {
    date: String,
    joins: i32,
    leaves: i32,
    downloads: u32,
}

async fn fetch_stats(ctx: Ctx<'_>, date: &str) -> Result<StatsResponse, Error> {
    let http_client = &ctx.data().http_client;
    let config = &ctx.data().config;
    let api_base = config.api_base.as_ref().context("API base URL not configured")?;
    let mut url = api_base.join("stats").context("failed to join URL path")?;
    url.query_pairs_mut().append_pair("date", date);

    http_client
        .get(url)
        .send()
        .await
        .context("Failed to send stats request")?
        .error_for_status()
        .context("API returned error status for stats request")?
        .json()
        .await
        .context("Failed to decode stats response")
}
