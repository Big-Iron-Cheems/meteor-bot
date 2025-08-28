use crate::{config::constants::EMBED_COLOR, Ctx, Error};
use poise::{serenity_prelude as serenity, CreateReply};
use serenity::{
    builder::{CreateActionRow, CreateButton, CreateEmbed},
    prelude::Mentionable,
};

/// Tells someone to read the FAQ
#[poise::command(slash_command, category = "Help")]
pub async fn faq(
    ctx: Ctx<'_>,
    #[description = "The member to tell to read the FAQ"] member: serenity::Member,
) -> Result<(), Error> {
    let embed = CreateEmbed::default()
        .title("Read the FAQ")
        .description(format!(
            "{} The FAQ answers your question, please read it.",
            member.mention()
        ))
        .color(EMBED_COLOR);

    let button = CreateButton::new_link("https://meteorclient.com/faq").label("FAQ");
    let action_row = CreateActionRow::Buttons(vec![button]);

    ctx.send(
        CreateReply::default()
            .embed(embed)
            .components(vec![action_row]),
    )
    .await?;

    Ok(())
}
