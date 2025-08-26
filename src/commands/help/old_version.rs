use crate::constants::{EMBED_COLOR, ERROR_COLOR};
use serenity::all::{ResolvedOption, ResolvedValue};
use serenity::builder::{
    CreateActionRow, CreateButton, CreateCommand, CreateCommandOption, CreateEmbed,
    CreateInteractionResponse, CreateInteractionResponseMessage,
};
use serenity::model::application::CommandOptionType;
use serenity::prelude::Mentionable;

pub fn register() -> CreateCommand {
    CreateCommand::new("old-versions")
        .description("Tells someone how to play on older versions of Minecraft")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::User,
                "member",
                "The member to tell how to play on older versions of Minecraft",
            )
            .required(true),
        )
}

pub async fn handle(options: &[ResolvedOption<'_>]) -> CreateInteractionResponse {
    let target_user = &options[0].value;

    let (content, embed, components, is_ephemeral) = if let ResolvedValue::User(user, _) =
        target_user
    {
        let embed = CreateEmbed::new()
                .title("Old Versions Guide")
                .description(format!(
                    "{} The old version guide explains how to play on older versions of Minecraft, please read it.",
                    user.mention()
                ))
                .color(EMBED_COLOR);

        let button =
            CreateButton::new_link("https://meteorclient.com/faq/old-versions").label("Guide");
        let action_row = CreateActionRow::Buttons(vec![button]);

        ("".to_string(), embed, vec![action_row], false)
    } else {
        let embed = CreateEmbed::new()
            .title("Error")
            .description("Could not find the specified user.")
            .color(ERROR_COLOR);

        ("".to_string(), embed, vec![], true)
    };

    let data = CreateInteractionResponseMessage::new()
        .content(content)
        .components(components)
        .embed(embed)
        .ephemeral(is_ephemeral);

    CreateInteractionResponse::Message(data)
}
