use crate::{config::constants::EMBED_COLOR, AppCtx, Ctx, Error};
use anyhow::Context;
use poise::{serenity_prelude as serenity, CreateReply};
use serenity::{
    model::{guild::Member, user::User},
    prelude::Mentionable,
    CreateEmbed,
};
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
            .map(|s| s.trim().to_lowercase())
            .map(|s| s == "true" || s == "yes" || s == "1")
            .unwrap_or(false);

        do_ban(app_ctx.into(), &member, response.reason, delete_messages).await?;
    } else {
        poise::Context::Application(app_ctx)
            .send(
                CreateReply::default()
                    .content("Ban cancelled.")
                    .ephemeral(true),
            )
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
async fn do_ban(
    ctx: Ctx<'_>,
    member: &Member,
    reason: Option<String>,
    delete_messages: bool,
) -> Result<(), Error> {
    let reason = reason.unwrap_or_else(|| "Reason unspecified".to_string());
    let delete_message_days = if delete_messages { 1 } else { 0 };

    match ctx
        .guild_id()
        .unwrap()
        .ban_with_reason(&ctx.http(), member.user.id, delete_message_days, &reason)
        .await
    {
        Ok(_) => {
            let embed = CreateEmbed::default()
                .title("Member Banned")
                .description(format!(
                    "{} has been banned.\nReason: {}\nDeleted messages: {}",
                    member.mention(),
                    reason,
                    if delete_messages {
                        "Yes (last 24h)"
                    } else {
                        "No"
                    }
                ))
                .color(EMBED_COLOR);

            ctx.send(CreateReply::default().embed(embed)).await?;
        }
        Err(e) => {
            error!("Error banning member {}: {}", member.user.id, e);
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
