use crate::{Data, Error};
use anyhow::Context;
use poise::serenity_prelude as serenity;
use serenity::User;

/// Notify backend of user leave
pub async fn member_remove_handler(data: &Data, user: &User) -> Result<(), Error> {
    let token = data
        .config
        .backend_token
        .as_ref()
        .expect("backend_token is always Some here");
    let api_base = data.config.api_base.as_ref().expect("api_base is always Some here");

    let url = format!("{}/discord/userLeft?id={}", api_base, user.id);

    data.http_client
        .post(&url)
        .header("Authorization", token)
        .send()
        .await
        .context("Failed to notify backend of user leave")?;

    Ok(())
}
