use crate::{config::constants::EMBED_COLOR, config::CONFIG, Context, Error};
use poise::{serenity_prelude as serenity, CreateReply};
use regex::Regex;
use reqwest::Client;
use serde::Deserialize;
use serenity::builder::CreateEmbed;
use std::sync::LazyLock;

/// Shows various stats about Meteor
#[poise::command(slash_command)]
pub async fn stats(
    ctx: Context<'_>,
    #[description = "The date to fetch the stats for (DD-MM-YYYY)"] date: Option<String>,
) -> Result<(), Error> {
    let date = match validate_date(date) {
        Ok(d) => d,
        Err(msg) => {
            ctx.send(CreateReply::default().content(msg).ephemeral(true))
                .await?;
            return Ok(());
        }
    };

    let stats = match fetch_stats(&ctx.data().http_client, &date).await {
        Ok(stats) => stats,
        Err(e) => {
            eprintln!("Error fetching stats for {}: {:?}", date, e);
            ctx.send(
                CreateReply::default()
                    .content("Failed to fetch stats for this date.")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    let gained = stats.joins as i32 - stats.leaves as i32;
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

fn validate_date(date: Option<String>) -> Result<String, &'static str> {
    static DATE_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^\d{2}-\d{2}-\d{4}$").expect("Invalid regex pattern"));

    match date {
        Some(d) if DATE_REGEX.is_match(&d) => Ok(d),
        Some(_) => Err("Invalid date format. Please use DD-MM-YYYY."),
        None => Ok(chrono::Local::now().format("%d-%m-%Y").to_string()),
    }
}

#[derive(Deserialize)]
struct StatsResponse {
    date: String,
    joins: u32,
    leaves: u32,
    downloads: u32,
}

async fn fetch_stats(http_client: &Client, date: &str) -> Result<StatsResponse, reqwest::Error> {
    let api_url = format!("{}/stats?date={}", CONFIG.api_base, date);

    let resp = http_client.get(&api_url).send().await?;
    resp.error_for_status()?.json().await
}
