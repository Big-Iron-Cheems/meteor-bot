use crate::{config::constants::EMBED_COLOR, Ctx, Error};
use poise::{serenity_prelude as serenity, CreateReply};
use serenity::{
    builder::{CreateActionRow, CreateButton, CreateEmbed},
    prelude::Mentionable,
};

/// Tells someone to read the installation guide
#[poise::command(slash_command, category = "Help")]
pub async fn installation(
    ctx: Ctx<'_>,
    #[description = "The member to tell to read the installation guide"] member: serenity::User,
) -> Result<(), Error> {
    let embed = CreateEmbed::default()
        .title("Read the Installation Guide")
        .description(format!(
            "{} The installation guide answers your question, please read it.",
            member.mention()
        ))
        .color(EMBED_COLOR);

    let button = CreateButton::new_link("https://meteorclient.com/faq/installation").label("Guide");
    let action_row = CreateActionRow::Buttons(vec![button]);

    ctx.send(
        CreateReply::default()
            .embed(embed)
            .components(vec![action_row]),
    )
    .await?;

    Ok(())
}
