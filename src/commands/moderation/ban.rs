use crate::{AppCtx, Ctx, Error, config::constants::EMBED_COLOR};
use anyhow::Context;
use poise::{CreateReply, serenity_prelude as serenity};
use serenity::{CreateEmbed, Member, Mentionable, User};
use tracing::error;

#[derive(poise::Modal)]
#[name = "Ban"]
struct BanModal {
    #[name = "Reason"]
    #[placeholder = "Enter the reason for the ban"]
    #[paragraph]
    reason: Option<String>,
    #[name = "Delete recent messages? (last 24h)"]
    #[placeholder = "yes or no"]
    delete_messages: Option<String>,
}

/// Context menu command for banning a user
#[poise::command(
    context_menu_command = "Ban",
    category = "Moderation",
    guild_only,
    default_member_permissions = "BAN_MEMBERS"
)]
pub async fn ban_menu(app_ctx: AppCtx<'_>, user: User) -> Result<(), Error> {
    let guild = app_ctx.guild().context("Not in a guild")?.to_owned();
    let member = guild.member(&app_ctx.serenity_context(), user.id).await?;

    let response: Option<BanModal> = poise::execute_modal(app_ctx, None, None).await?;
    if let Some(response) = response {
        let delete_messages = response
            .delete_messages
            .is_some_and(|s| matches!(s.trim().to_lowercase().as_str(), "true" | "yes" | "1"));

        do_ban(app_ctx.into(), &member, response.reason, delete_messages).await?;
    } else {
        poise::Context::Application(app_ctx)
            .send(CreateReply::default().content("Ban cancelled.").ephemeral(true))
            .await?;
    }
    Ok(())
}

/// Bans a member
#[poise::command(
    slash_command,
    category = "Moderation",
    guild_only,
    default_member_permissions = "BAN_MEMBERS"
)]
pub async fn ban(
    ctx: Ctx<'_>,
    #[description = "The member to ban"] member: Member,
    #[description = "The reason for the ban"] reason: Option<String>,
    #[description = "Delete recent messages?"] delete_messages: bool,
) -> Result<(), Error> {
    do_ban(ctx, &member, reason, delete_messages).await?;
    Ok(())
}

/// Shared ban logic
async fn do_ban(ctx: Ctx<'_>, member: &Member, reason: Option<String>, delete_messages: bool) -> Result<(), Error> {
    let reason = reason.unwrap_or_else(|| "Reason unspecified".to_string());
    let delete_message_days = u8::from(delete_messages);

    match ctx
        .guild_id()
        .context("guild_id missing in guild-only command")?
        .ban_with_reason(ctx, member.user.id, delete_message_days, &reason)
        .await
    {
        Ok(()) => {
            let embed = CreateEmbed::default()
                .title("Member Banned")
                .description(format!(
                    "{} has been banned.\nReason: {reason}\nDeleted messages: {}",
                    member.mention(),
                    if delete_messages { "Yes (last 24h)" } else { "No" }
                ))
                .color(EMBED_COLOR);

            ctx.send(CreateReply::default().embed(embed)).await?;
        }
        Err(e) => {
            error!("Error banning member {}: {e}", member.user.id);
            ctx.send(
                CreateReply::default()
                    .content("An error occurred while banning the member.")
                    .ephemeral(true),
            )
            .await?;
        }
    }
    Ok(())
}
