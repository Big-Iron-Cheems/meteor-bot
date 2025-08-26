use crate::{constants::EMBED_COLOR, Context, Error};
use poise::{serenity_prelude as serenity, CreateReply};
use serenity::{model::guild::Member, prelude::Mentionable, CreateEmbed};

/// Bans a member
#[poise::command(slash_command, guild_only, default_member_permissions = "BAN_MEMBERS")]
pub async fn ban(
    ctx: Context<'_>,
    #[description = "The member to ban"] member: Member,
    #[description = "The reason for the ban"] reason: Option<String>,
    #[description = "Delete recent messages?"] delete_messages: bool,
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
            eprintln!("Error banning member {}: {:?}", member.user.id, e);
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
