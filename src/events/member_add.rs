use crate::{config::CONFIG, Data, Error};
use poise::serenity_prelude as serenity;
use serenity::Member;

/// Notify backend of user join
pub async fn member_add_handler(data: &Data, member: &Member) -> Result<(), Error> {
    let Some(token) = &CONFIG.backend_token else {
        return Ok(());
    };

    let Some(api_base) = &CONFIG.api_base else {
        return Ok(());
    };

    let url = format!("{}/discord/userJoined?id={}", api_base, member.user.id);

    data.http_client
        .post(&url)
        .header("Authorization", token)
        .send()
        .await?;

    Ok(())
}
