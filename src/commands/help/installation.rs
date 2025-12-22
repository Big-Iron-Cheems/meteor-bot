use crate::{Ctx, Error, config::constants::EMBED_COLOR};
use poise::{CreateReply, serenity_prelude as serenity};
use serenity::{
    builder::{CreateActionRow, CreateButton, CreateEmbed},
    model::user::User,
    prelude::Mentionable,
};

/// Tells someone to read the installation guide
#[poise::command(slash_command, category = "Help")]
pub async fn installation(
    ctx: Ctx<'_>,
    #[description = "User to direct to the installation guide"] user: User,
) -> Result<(), Error> {
    let embed = CreateEmbed::default()
        .title("Read the Installation Guide")
        .description(format!(
            "{} The installation guide answers your question, please read it.",
            user.mention()
        ))
        .color(EMBED_COLOR);

    let button = CreateButton::new_link("https://meteorclient.com/faq/installation").label("Guide");
    let action_row = CreateActionRow::Buttons(vec![button]);

    ctx.send(CreateReply::default().embed(embed).components(vec![action_row]))
        .await?;

    Ok(())
}
