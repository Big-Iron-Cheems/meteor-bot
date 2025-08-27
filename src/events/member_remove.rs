use crate::{config::CONFIG, Data, Error};
use poise::serenity_prelude as serenity;
use serenity::User;

/// Notify backend of user leave
pub async fn member_remove_handler(data: &Data, user: &User) -> Result<(), Error> {
    if CONFIG.backend_token.is_empty() {
        return Ok(());
    }

    let url = format!("{}/discord/userLeft?id={}", CONFIG.api_base, user.id);

    data.http_client
        .post(&url)
        .header("Authorization", &CONFIG.backend_token)
        .send()
        .await?;

    Ok(())
}
