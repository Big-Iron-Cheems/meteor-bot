use crate::{constants::EMBED_COLOR, Context, Error};
use poise::{serenity_prelude as serenity, CreateReply};
use serenity::{
    builder::{CreateActionRow, CreateButton, CreateEmbed},
    prelude::Mentionable,
};

/// Tells someone how to play on older versions of Minecraft
#[poise::command(slash_command, rename = "old-versions")]
pub async fn old_versions(
    ctx: Context<'_>,
    #[description = "The member to tell how to play on older versions of Minecraft"]
    member: serenity::User,
) -> Result<(), Error> {
    let embed = CreateEmbed::default()
        .title("Old Versions Guide")
        .description(format!(
            "{} The old version guide explains how to play on older versions of Minecraft, please read it.",
            member.mention()
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
