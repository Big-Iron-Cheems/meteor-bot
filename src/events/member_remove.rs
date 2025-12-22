use crate::{Data, Error};
use anyhow::Context;
use poise::serenity_prelude as serenity;
use serenity::User;

#[derive(serde::Serialize)]
struct UserLeftParams {
    id: serenity::UserId,
}

/// Notify backend of user leave
pub async fn member_remove_handler(data: &Data, user: &User) -> Result<(), Error> {
    let token = data
        .config
        .backend_token
        .as_ref()
        .expect("backend_token is always Some here");
    let api_base = data.config.api_base.as_ref().expect("api_base is always Some here");

    data.http_client
        .post(api_base.join("discord/userLeft").expect("failed to join URL path"))
        .query(&UserLeftParams { id: user.id })
        .header("Authorization", token)
        .send()
        .await
        .context("Failed to notify backend of user leave")?;

    Ok(())
}
