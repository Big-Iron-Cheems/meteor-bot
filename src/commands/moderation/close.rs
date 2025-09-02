use crate::{config::constants::EMBED_COLOR, Ctx, Error};
use anyhow::Context;
use poise::{serenity_prelude as serenity, CreateReply};
use serenity::{
    builder::{CreateEmbed, EditThread},
    model::channel::ChannelType,
};
use tracing::error;

/// Locks the current forum post
#[poise::command(
    slash_command,
    category = "Moderation",
    guild_only,
    default_member_permissions = "MANAGE_THREADS",
    check = "is_thread_channel"
)]
pub async fn close(ctx: Ctx<'_>) -> Result<(), Error> {
    let mut channel = ctx.guild_channel().await.context("Failed to fetch channel")?;

    let embed = CreateEmbed::default()
        .title("Thread Closed")
        .description("Thread has been locked and archived.")
        .color(EMBED_COLOR);

    ctx.send(CreateReply::default().embed(embed)).await?;

    match channel
        .edit_thread(&ctx.http(), EditThread::default().locked(true).archived(true))
        .await
    {
        Ok(_) => {
            // Thread successfully closed - embed was already sent
        }
        Err(e) => {
            error!("Error locking thread {}: {}", channel.id, e);
            ctx.send(
                CreateReply::default()
                    .content("Failed to lock/archive: this thread may already be archived.")
                    .ephemeral(true),
            )
            .await?;
        }
    }

    Ok(())
}

/// Ensure the command is used in a thread channel, otherwise send an error message.
async fn is_thread_channel(ctx: Ctx<'_>) -> Result<bool, Error> {
    let channel = match ctx.guild_channel().await {
        Some(c) => c,
        None => return Ok(false),
    };

    match channel.kind {
        ChannelType::PublicThread | ChannelType::PrivateThread | ChannelType::NewsThread => Ok(true),
        _ => {
            ctx.send(
                CreateReply::default()
                    .content("This command can only be used inside a thread.")
                    .ephemeral(true),
            )
            .await?;
            Ok(false)
        }
    }
}
