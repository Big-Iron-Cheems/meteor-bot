use crate::{Data, Error};
use anyhow::Context;
use poise::serenity_prelude as serenity;
use serenity::Member;
use url::Url;

#[derive(serde::Serialize)]
struct UserJoinedParams {
    id: serenity::UserId,
}

/// Notify backend of user join
pub async fn member_add_handler(data: &Data, member: &Member, token: &str, api_base: &Url) -> Result<(), Error> {
    data.http_client
        .post(api_base.join("discord/userJoined").context("failed to join URL path")?)
        .query(&UserJoinedParams { id: member.user.id })
        .header("Authorization", token)
        .send()
        .await
        .context("Failed to notify backend of user join")?;

    Ok(())
}
