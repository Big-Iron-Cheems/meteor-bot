use crate::{config::constants::EMBED_COLOR, AppCtx, Ctx, Error};
use anyhow::Context;
use chrono::{Duration, Utc};
use poise::{serenity_prelude as serenity, CreateReply};
use serenity::{
    model::{guild::Member, user::User}, prelude::Mentionable,
    CreateEmbed,
    EditMember,
};
use tracing::error;

/// Discord enforces a maximum timeout duration of 28 days
const TIMEOUT_MAX_DAYS: i64 = 28;

#[derive(poise::Modal)]
#[name = "Mute"]
struct MuteModal {
    #[name = "Duration (e.g., 1s, 1m, 1h, 1d, 1w)"]
    #[placeholder = "1h"]
    duration: String,
    #[name = "Reason"]
    #[placeholder = "Enter the reason for the mute"]
    #[paragraph]
    reason: Option<String>,
}

/// Context menu command for muting a user
#[poise::command(
    context_menu_command = "Mute",
    category = "Moderation",
    guild_only,
    default_member_permissions = "MODERATE_MEMBERS"
)]
pub async fn mute_menu(app_ctx: AppCtx<'_>, user: User) -> Result<(), Error> {
    let guild = app_ctx.guild().context("Not in a guild")?.to_owned();
    let member = guild.member(&app_ctx.serenity_context(), user.id).await?;

    let response: Option<MuteModal> = poise::execute_modal(app_ctx, None, None).await?;
    if let Some(response) = response {
        let duration = response
            .duration
            .parse::<humantime::Duration>()
            .map_err(|_| anyhow::anyhow!("Invalid duration format"))?;
        do_mute(app_ctx.into(), &member, duration, response.reason).await?;
    } else {
        poise::Context::Application(app_ctx)
            .send(
                CreateReply::default()
                    .content("Mute cancelled.")
                    .ephemeral(true),
            )
            .await?;
    }
    Ok(())
}

/// Mutes a member
#[poise::command(
    slash_command,
    category = "Moderation",
    guild_only,
    default_member_permissions = "MODERATE_MEMBERS"
)]
pub async fn mute(
    ctx: Ctx<'_>,
    #[description = "The member to mute"] member: Member,
    #[description = "The duration of the mute (e.g., 1s, 1m, 1h, 1d, 1w)"]
    duration: humantime::Duration,
    #[description = "The reason for the mute"] reason: Option<String>,
) -> Result<(), Error> {
    do_mute(ctx, &member, duration, reason).await?;
    Ok(())
}

/// Shared mute logic
async fn do_mute(
    ctx: Ctx<'_>,
    member: &Member,
    duration: humantime::Duration,
    reason: Option<String>,
) -> Result<(), Error> {
    let chrono_dur =
        Duration::from_std(*duration).map_err(|_| anyhow::anyhow!("Invalid duration"))?;

    // validate against the max
    if chrono_dur > Duration::days(TIMEOUT_MAX_DAYS) {
        ctx.send(
            CreateReply::default()
                .content(format!(
                    "Invalid duration: exceeds the maximum allowed value of {} days.",
                    TIMEOUT_MAX_DAYS
                ))
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    let reason = reason.unwrap_or_else(|| "Reason unspecified".to_string());

    if let Some(timeout_until) = member.communication_disabled_until {
        if timeout_until > Utc::now().into() {
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

    let mute_until = Utc::now() + chrono_dur;

    match ctx
        .guild_id()
        .unwrap()
        .edit_member(
            &ctx.http(),
            member.user.id,
            EditMember::new()
                .disable_communication_until_datetime(mute_until.into())
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
                    humantime::format_duration(duration.into())
                ))
                .color(EMBED_COLOR);

            ctx.send(CreateReply::default().embed(embed)).await?;
        }
        Err(e) => {
            error!("Error muting member {}: {}", member.user.id, e);
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
