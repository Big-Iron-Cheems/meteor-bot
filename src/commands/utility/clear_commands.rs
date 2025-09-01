use crate::{Ctx, Error};
use poise::CreateReply;

/// Clear all guild commands (admin only)
#[poise::command(slash_command, category = "Utility", owners_only)]
pub async fn clear_guild_commands(ctx: Ctx<'_>) -> Result<(), Error> {
    if let Some(guild_id) = ctx.guild_id() {
        ctx.http()
            .create_guild_commands(guild_id, &Vec::<()>::new())
            .await?;

        ctx.send(
            CreateReply::default()
                .content("Cleared all guild commands! Only global commands remain.")
                .ephemeral(true),
        )
        .await?;
    } else {
        ctx.send(
            CreateReply::default()
                .content("This command must be used in a guild")
                .ephemeral(true),
        )
        .await?;
    }
    Ok(())
}
