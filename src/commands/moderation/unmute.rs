use crate::{AppCtx, Ctx, Error, config::constants::EMBED_COLOR};
use anyhow::Context;
use poise::{CreateReply, serenity_prelude as serenity};
use serenity::{
    CreateEmbed, EditMember,
    model::{guild::Member, user::User},
    prelude::Mentionable,
};
use tracing::error;

#[derive(poise::Modal)]
#[name = "Unmute"]
struct UnmuteModal {
    #[name = "Reason"]
    #[placeholder = "Enter the reason for the unmute"]
    #[paragraph]
    reason: Option<String>,
}

/// Context menu command for unmuting a user
#[poise::command(
    context_menu_command = "Unmute",
    category = "Moderation",
    guild_only,
    default_member_permissions = "MODERATE_MEMBERS"
)]
pub async fn unmute_menu(app_ctx: AppCtx<'_>, user: User) -> Result<(), Error> {
    let guild = app_ctx.guild().context("Not in a guild")?.to_owned();
    let member = guild.member(&app_ctx.serenity_context(), user.id).await?;

    let response: Option<UnmuteModal> = poise::execute_modal(app_ctx, None, None).await?;
    if let Some(response) = response {
        do_unmute(app_ctx.into(), &member, response.reason).await?;
    } else {
        poise::Context::Application(app_ctx)
            .send(CreateReply::default().content("Unmute cancelled.").ephemeral(true))
            .await?;
    }
    Ok(())
}

/// Unmutes a member
#[poise::command(
    slash_command,
    category = "Moderation",
    guild_only,
    default_member_permissions = "MODERATE_MEMBERS"
)]
pub async fn unmute(
    ctx: Ctx<'_>,
    #[description = "The member to unmute"] member: Member,
    #[description = "The reason for the unmute"] reason: Option<String>,
) -> Result<(), Error> {
    do_unmute(ctx, &member, reason).await?;
    Ok(())
}

/// Shared unmute logic
async fn do_unmute(ctx: Ctx<'_>, member: &Member, reason: Option<String>) -> Result<(), Error> {
    if member.communication_disabled_until.is_none() {
        ctx.send(
            CreateReply::default()
                .content(format!("{} is not currently muted.", member.mention()))
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    let reason = reason.unwrap_or_else(|| "Reason unspecified".to_string());

    match ctx
        .guild_id()
        .unwrap()
        .edit_member(
            ctx,
            member.user.id,
            EditMember::new().enable_communication().audit_log_reason(&reason),
        )
        .await
    {
        Ok(_) => {
            let embed = CreateEmbed::default()
                .title("Member Unmuted")
                .description(format!("{} has been unmuted.", member.mention()))
                .color(EMBED_COLOR);

            ctx.send(CreateReply::default().embed(embed)).await?;
        }
        Err(e) => {
            error!("Error unmuting member {}: {e}", member.user.id);
            ctx.send(
                CreateReply::default()
                    .content("An error occurred while unmuting the member.")
                    .ephemeral(true),
            )
            .await?;
        }
    }

    Ok(())
}
