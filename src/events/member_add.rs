use crate::{Data, Error};
use anyhow::Context;
use poise::serenity_prelude as serenity;
use serenity::Member;

/// Notify backend of user join
pub async fn member_add_handler(data: &Data, member: &Member) -> Result<(), Error> {
    let (Some(token), Some(api_base)) = (&data.config.backend_token, &data.config.api_base) else {
        return Ok(());
    };

    let url = format!("{}/discord/userJoined?id={}", api_base, member.user.id);

    data.http_client
        .post(&url)
        .header("Authorization", token)
        .send()
        .await
        .context("Failed to notify backend of user join")?;

    Ok(())
}
