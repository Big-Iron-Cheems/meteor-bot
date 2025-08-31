use crate::{config::constants::EMBED_COLOR, Ctx, Error};
use poise::{serenity_prelude as serenity, CreateReply};
use serde::Deserialize;
use serde_json::Value;
use serenity::builder::CreateEmbed;
use std::collections::HashMap;

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
        Err(LinkError::MissingAPIBase) => {
            eprintln!("API base URL is not set. Cannot link Discord account.");
            ctx.send(
                CreateReply::default()
                    .content("Failed to link your Discord account. Please try again later.")
                    .ephemeral(true),
            )
            .await?;
        }
        Err(LinkError::MissingBackendToken) => {
            eprintln!("Backend token is not set. Cannot link Discord account.");
            ctx.send(
                CreateReply::default()
                    .content("Failed to link your Discord account. Please try again later.")
                    .ephemeral(true),
            )
            .await?;
        }
        Err(LinkError::InvalidToken) => {
            ctx.send(
                CreateReply::default()
                    .content("Failed to link your Discord account. Try generating a new token by refreshing the account page and clicking the link button again.")
                    .ephemeral(true),
            )
            .await?;
        }
        Err(LinkError::RequestFailed(e)) => {
            eprintln!(
                "Failed to link Discord account for user {}: {:?}",
                user_id, e
            );
            ctx.send(
                CreateReply::default()
                    .content("Failed to link your Discord account. Please try again later.")
                    .ephemeral(true),
            )
            .await?;
        }
        Err(LinkError::DecodeFailed(e)) => {
            eprintln!("Failed to decode response for user {}: {:?}", user_id, e);
            ctx.send(
                CreateReply::default()
                    .content("Failed to decode the response.")
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

#[derive(Debug)]
enum LinkError {
    MissingAPIBase,
    MissingBackendToken,
    InvalidToken,
    RequestFailed(reqwest::Error),
    DecodeFailed(reqwest::Error),
}

async fn link_discord_account(ctx: &Ctx<'_>, user_id: &str, token: &str) -> Result<(), LinkError> {
    let Some(api_base) = &ctx.data().config.api_base else {
        return Err(LinkError::MissingAPIBase);
    };

    let api_url = format!("{}/account/linkDiscord", api_base);
    let form_data = [("id", user_id), ("token", token)];

    let backend_token = ctx
        .data()
        .config
        .backend_token
        .as_ref()
        .ok_or(LinkError::MissingBackendToken)?;

    let resp = ctx
        .data()
        .http_client
        .post(&api_url)
        .header("Authorization", backend_token)
        .form(&form_data)
        .send()
        .await
        .map_err(LinkError::RequestFailed)?;

    let json_response = resp
        .json::<ApiResponse>()
        .await
        .map_err(LinkError::DecodeFailed)?;

    if json_response.data.contains_key("error") {
        return Err(LinkError::InvalidToken);
    }

    Ok(())
}
