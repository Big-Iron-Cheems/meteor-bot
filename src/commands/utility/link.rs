use crate::{Ctx, Error, config::constants::EMBED_COLOR};
use anyhow::Context;
use poise::{CreateReply, serenity_prelude as serenity};
use serenity::builder::CreateEmbed;
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

            ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
        }
        Err(e) => {
            error!("Failed to link Discord account for user {user_id}: {e}");

            ctx.send(CreateReply::default().content(e.to_string()).ephemeral(true))
                .await?;
        }
    }

    Ok(())
}

#[derive(serde::Serialize)]
struct LinkDiscordParams<'a> {
    id: &'a str,
    token: &'a str,
}

#[derive(serde::Deserialize)]
struct BackendError {
    error: String,
}

async fn link_discord_account(ctx: &Ctx<'_>, user_id: &str, token: &str) -> Result<(), Error> {
    let api_base = ctx
        .data()
        .config
        .api_base
        .as_ref()
        .context("API base URL not configured")?;

    let backend_token = ctx
        .data()
        .config
        .backend_token
        .as_ref()
        .context("Backend token not configured")?;

    let resp = ctx
        .data()
        .http_client
        .post(
            api_base
                .join("account/linkDiscord")
                .context("failed to join URL path")?,
        )
        .header("Authorization", backend_token)
        .query(&LinkDiscordParams { id: user_id, token })
        .send()
        .await
        .context("Failed to send link request")?;

    if resp.status().is_success() {
        return Ok(());
    }

    let backend_error = resp
        .json::<BackendError>()
        .await
        .context("Failed to decode backend error response")?;

    anyhow::bail!(backend_error.error);
}
