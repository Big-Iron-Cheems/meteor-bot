use crate::{config::constants::EMBED_COLOR, Context, Error};
use chrono::{DateTime, Utc};
use poise::{serenity_prelude as serenity, CreateReply};
use serenity::{
    model::guild::Member, model::Permissions, prelude::Mentionable, CreateEmbed, EditMember,
};
use std::time::{Duration, SystemTime};

/// Discord enforces a maximum timeout duration of 28 days (4 weeks)
const TIMEOUT_MAX_SECS: u64 = 28 * 24 * 60 * 60;

/// Mutes a member
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "MODERATE_MEMBERS"
)]
pub async fn mute(
    ctx: Context<'_>,
    #[description = "The member to mute"] member: Member,
    #[description = "The duration of the mute (e.g., 1s, 1m, 1h, 1d, 1w)"]
    duration: humantime::Duration,
    #[description = "The reason for the mute"] reason: Option<String>,
) -> Result<(), Error> {
    let duration_parsed: Duration = duration.into();
    if duration_parsed.as_secs() > TIMEOUT_MAX_SECS {
        ctx.send(
            CreateReply::default()
                .content("Invalid duration: exceeds the maximum allowed value of 4 weeks.")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    let reason = reason.unwrap_or_else(|| "Reason unspecified".to_string());

    if let Some(timeout_until) = member.communication_disabled_until {
        let now = Utc::now();
        if timeout_until > now.into() {
            ctx.send(
                CreateReply::default()
                    .content(format!(
                        "{} is already muted until {}.",
                        member.mention(),
                        timeout_until,
                    ))
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    }

    let channel = ctx.guild_channel().await.ok_or("Failed to fetch channel")?;
    let member_permissions = ctx
        .guild()
        .ok_or("Failed to fetch guild")?
        .user_permissions_in(&channel, &member);
    if member_permissions.contains(Permissions::MODERATE_MEMBERS) {
        ctx.send(
            CreateReply::default()
                .content("You do not have the required permissions to mute this member.")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    let mute_until = SystemTime::now() + duration_parsed;
    let mute_timestamp: DateTime<Utc> = mute_until.into();

    match ctx
        .guild_id()
        .unwrap()
        .edit_member(
            &ctx.http(),
            member.user.id,
            EditMember::new()
                .disable_communication_until_datetime(mute_timestamp.into())
                .audit_log_reason(&reason),
        )
        .await
    {
        Ok(_) => {
            let embed = CreateEmbed::default()
                .title("Member Muted")
                .description(format!(
                    "Muted {} for {}.",
                    member.user.mention(),
                    humantime::format_duration(duration_parsed)
                ))
                .color(EMBED_COLOR);

            ctx.send(CreateReply::default().embed(embed)).await?;
        }
        Err(e) => {
            eprintln!("Error muting member {}: {:?}", member.user.id, e);
            ctx.send(
                CreateReply::default()
                    .content("An error occurred while muting the member.")
                    .ephemeral(true),
            )
            .await?;
        }
    }

    Ok(())
}
