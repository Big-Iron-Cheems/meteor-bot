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
            if data.config.backend_token.is_some() && data.config.api_base.is_some() {
                log_err("member_add_handler", member_add::member_add_handler(data, new_member)).await;
            }
        }
        FullEvent::GuildMemberRemoval { user, .. } => {
            if data.config.backend_token.is_some() && data.config.api_base.is_some() {
                log_err(
                    "member_remove_handler",
                    member_remove::member_remove_handler(data, user),
                )
                .await;
            }
        }
        FullEvent::Message { new_message } => {
            if data.config.guild_id.is_some() {
                log_err("message_handler", message::message_handler(ctx, data, new_message)).await;
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
    F: Future<Output = Result<T, Error>>,
{
    if let Err(e) = fut.await {
        error!("Error in {label}: {e}");
    }
}
