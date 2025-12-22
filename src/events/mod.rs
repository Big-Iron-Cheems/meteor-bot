mod member_add;
mod member_remove;
mod message;
mod ready;

use crate::{Data, Error};
use poise::{FrameworkContext, serenity_prelude as serenity};
use serenity::{Context, all::FullEvent};
use tracing::{error, info};

/// Main event handler
pub async fn event_handler(
    ctx: &Context,
    event: &FullEvent,
    _framework: FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        FullEvent::Ready { data_about_bot, .. } => {
            info!("Logged in as {}", data_about_bot.user.name);
            log_err("ready_handler", ready::ready_handler(ctx, data)).await;
        }
        FullEvent::GuildMemberAddition { new_member } => {
            if let (Some(token), Some(api_base)) = (data.config.backend_token.as_ref(), data.config.api_base.as_ref()) {
                log_err(
                    "member_add_handler",
                    member_add::member_add_handler(data, new_member, token, api_base),
                )
                .await;
            }
        }
        FullEvent::GuildMemberRemoval { user, .. } => {
            if let (Some(token), Some(api_base)) = (data.config.backend_token.as_ref(), data.config.api_base.as_ref()) {
                log_err(
                    "member_remove_handler",
                    member_remove::member_remove_handler(data, user, token, api_base),
                )
                .await;
            }
        }
        FullEvent::Message { new_message } => {
            if let Some(guild_id) = data.config.guild_id {
                log_err(
                    "message_handler",
                    message::message_handler(ctx, data, new_message, guild_id),
                )
                .await;
            }
        }
        _ => {
            // Ignore other events
        }
    }

    Ok(())
}

/// Log error from event handlers
async fn log_err<F, T>(label: &str, fut: F)
where
    F: IntoFuture<Output = Result<T, Error>>,
{
    if let Err(e) = fut.into_future().await {
        error!("Error in {label}: {e}");
    }
}
