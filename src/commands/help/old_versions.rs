use crate::{config::constants::EMBED_COLOR, Ctx, Error};
use poise::{serenity_prelude as serenity, CreateReply};
use serenity::{
    builder::{CreateActionRow, CreateButton, CreateEmbed},
    model::user::User,
    prelude::Mentionable,
};

/// Tells someone how to play on older versions of Minecraft
#[poise::command(slash_command, category = "Help", rename = "old-versions")]
pub async fn old_versions(
    ctx: Ctx<'_>,
    #[description = "User to help with older Minecraft versions"] user: User,
) -> Result<(), Error> {
    let embed = CreateEmbed::default()
        .title("Old Versions Guide")
        .description(format!(
            "{} The old version guide explains how to play on older versions of Minecraft, please read it.",
            user.mention()
        ))
        .color(EMBED_COLOR);

    let button = CreateButton::new_link("https://meteorclient.com/faq/old-versions").label("Guide");
    let action_row = CreateActionRow::Buttons(vec![button]);

    ctx.send(
        CreateReply::default()
            .embed(embed)
            .components(vec![action_row]),
    )
    .await?;

    Ok(())
}
