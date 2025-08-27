use crate::{config::CONFIG, Data, Error};
use poise::serenity_prelude as serenity;
use serenity::Member;

/// Notify backend of user join
pub async fn member_add_handler(data: &Data, member: &Member) -> Result<(), Error> {
    if CONFIG.backend_token.is_empty() {
        return Ok(());
    }

    let url = format!(
        "{}/discord/userJoined?id={}",
        CONFIG.api_base, member.user.id
    );

    data.http_client
        .post(&url)
        .header("Authorization", &CONFIG.backend_token)
        .send()
        .await?;

    Ok(())
}
