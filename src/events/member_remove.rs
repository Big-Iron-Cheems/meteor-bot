use crate::{Data, Error};
use anyhow::Context;
use poise::serenity_prelude as serenity;
use serenity::User;
use url::Url;

#[derive(serde::Serialize)]
struct UserLeftParams {
    id: serenity::UserId,
}

/// Notify backend of user leave
pub async fn member_remove_handler(data: &Data, user: &User, token: &str, api_base: &Url) -> Result<(), Error> {
    data.http_client
        .post(api_base.join("discord/userLeft").context("failed to join URL path")?)
        .query(&UserLeftParams { id: user.id })
        .header("Authorization", token)
        .send()
        .await
        .context("Failed to notify backend of user leave")?;

    Ok(())
}
