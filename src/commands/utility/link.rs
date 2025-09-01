use crate::{config::constants::EMBED_COLOR, Ctx, Error};
use anyhow::Context;
use poise::{serenity_prelude as serenity, CreateReply};
use serde::Deserialize;
use serde_json::Value;
use serenity::builder::CreateEmbed;
use std::collections::HashMap;
use tracing::error;

/// Links your Discord account to your Meteor account
#[poise::command(slash_command, category = "Utility", dm_only)]
pub async fn link(
    ctx: Ctx<'_>,
    #[description = "The token generated on the Meteor website"] token: String,
) -> Result<(), Error> {
    if token.trim().is_empty() {
        ctx.send(
            CreateReply::default()
                .content("You must provide a valid token.")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    let user_id = ctx.author().id.to_string();

    match link_discord_account(&ctx, &user_id, &token).await {
        Ok(()) => {
            let embed = CreateEmbed::default()
                .title("Account Linked")
                .description("Successfully linked your Discord account.")
                .color(EMBED_COLOR);

            ctx.send(CreateReply::default().embed(embed).ephemeral(true))
                .await?;
        }
        Err(e) => {
            error!(
                "Failed to link Discord account for user {}: {:#}",
                user_id, e
            );

            ctx.send(
                CreateReply::default()
                    .content(e.to_string())
                    .ephemeral(true),
            )
            .await?;
        }
    }

    Ok(())
}

#[derive(Deserialize)]
struct ApiResponse {
    #[serde(flatten)]
    data: HashMap<String, Value>,
}

async fn link_discord_account(ctx: &Ctx<'_>, user_id: &str, token: &str) -> Result<(), Error> {
    let api_base = ctx
        .data()
        .config
        .api_base
        .as_ref()
        .context("API base URL is not set. Cannot link Discord account.")?;
    let api_url = format!("{}/account/linkDiscord", api_base);
    let form_data = [("id", user_id), ("token", token)];
    let backend_token = ctx
        .data()
        .config
        .backend_token
        .as_ref()
        .context("Backend token is not set. Cannot link Discord account.")?;
    let resp = ctx
        .data()
        .http_client
        .post(&api_url)
        .header("Authorization", backend_token)
        .form(&form_data)
        .send()
        .await
        .context("Failed to send request to link Discord account.")?;
    let json_response = resp
        .json::<ApiResponse>()
        .await
        .context("Failed to decode response from backend.")?;
    if json_response.data.contains_key("error") {
        anyhow::bail!(
            "Failed to link your Discord account. Try generating a new token by refreshing the account page and clicking the link button again."
        );
    }
    Ok(())
}
