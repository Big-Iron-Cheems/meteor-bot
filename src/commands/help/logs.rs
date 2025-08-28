use crate::{config::constants::EMBED_COLOR, Ctx, Error};
use poise::{serenity_prelude as serenity, CreateReply};
use serenity::{
    builder::{CreateActionRow, CreateButton, CreateEmbed},
    prelude::Mentionable,
};

/// Tells someone how to find the Minecraft logs
#[poise::command(slash_command, category = "Help")]
pub async fn logs(
    ctx: Ctx<'_>,
    #[description = "The member to tell how to find the Minecraft logs"] member: serenity::User,
) -> Result<(), Error> {
    let embed = CreateEmbed::default()
        .title("Find the Minecraft Logs")
        .description(format!(
            "{} The logs guide explains how to find and share your Minecraft logs, please read it.",
            member.mention()
        ))
        .color(EMBED_COLOR);

    let button = CreateButton::new_link("https://meteorclient.com/faq/getting-log").label("Guide");
    let action_row = CreateActionRow::Buttons(vec![button]);

    ctx.send(
        CreateReply::default()
            .embed(embed)
            .components(vec![action_row]),
    )
    .await?;

    Ok(())
}
