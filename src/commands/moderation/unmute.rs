use crate::{constants::EMBED_COLOR, Context, Error};
use poise::{serenity_prelude as serenity, CreateReply};
use serenity::{builder::CreateEmbed, model::guild::Member, prelude::Mentionable, EditMember};

/// Unmutes a member
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "MODERATE_MEMBERS"
)]
pub async fn unmute(
    ctx: Context<'_>,
    #[description = "The member to unmute"] member: Member,
) -> Result<(), Error> {
    if member.communication_disabled_until.is_none() {
        ctx.send(
            CreateReply::default()
                .content(format!("{} is not currently muted.", member.mention()))
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    match ctx
        .guild_id()
        .unwrap()
        .edit_member(
            &ctx.http(),
            member.user.id,
            EditMember::new().enable_communication(),
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
            eprintln!("Error unmuting member {}: {:?}", member.user.id, e);
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
