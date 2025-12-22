use crate::{Data, Error};
use anyhow::Context;
use poise::serenity_prelude as serenity;
use serenity::Member;

#[derive(serde::Serialize)]
struct UserJoinedParams {
    id: serenity::UserId,
}

/// Notify backend of user join
pub async fn member_add_handler(data: &Data, member: &Member) -> Result<(), Error> {
    let token = data
        .config
        .backend_token
        .as_ref()
        .expect("backend_token is always Some here");
    let api_base = data.config.api_base.as_ref().expect("api_base is always Some here");

    data.http_client
        .post(api_base.join("discord/userJoined").expect("failed to join URL path"))
        .query(&UserJoinedParams { id: member.user.id })
        .header("Authorization", token)
        .send()
        .await
        .context("Failed to notify backend of user join")?;

    Ok(())
}
