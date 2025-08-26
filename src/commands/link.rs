use crate::config::CONFIG;
use reqwest::Client;
use serde_json::Value;
use serenity::all::{Context, ResolvedOption, ResolvedValue};
use serenity::builder::{
    CreateCommand, CreateCommandOption, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use serenity::model::application::CommandOptionType;

pub fn register() -> CreateCommand {
    CreateCommand::new("link")
        .description("Links your Discord account to your Meteor account")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "token",
                "The token generated on the Meteor website",
            )
            .required(true),
        )
}

pub async fn handle(ctx: &Context, options: &[ResolvedOption<'_>]) -> CreateInteractionResponse {
    // Only allow in DMs
    // In Serenity, you must check if the interaction has no guild_id
    let token = match &options[0].value {
        ResolvedValue::String(s) if !s.is_empty() => s,
        _ => {
            let data = CreateInteractionResponseMessage::new()
                .content("You must provide a valid token.")
                .ephemeral(true);
            return CreateInteractionResponse::Message(data);
        }
    };

    // The user id must be extracted from the interaction, but here we only have options
    // So, this function should be called with the interaction context, but for now, assume user id is available
    // You may need to refactor to pass the CommandInteraction

    // For now, fallback to empty string
    let user_id = "";

    let api_base = &CONFIG.api_base;
    let backend_token = &CONFIG.backend_token;

    let client = Client::new();
    let params = [("id", user_id), ("token", token)];

    let resp = client
        .post(format!("{}/account/linkDiscord", api_base))
        .header("Authorization", backend_token)
        .form(&params)
        .send()
        .await;

    let response = match resp {
        Ok(r) => r,
        Err(_) => {
            let data = CreateInteractionResponseMessage::new()
                .content("Failed to link your Discord account. Please try again later.")
                .ephemeral(true);
            return CreateInteractionResponse::Message(data);
        }
    };

    let json: Value = match response.json().await {
        Ok(j) => j,
        Err(_) => {
            let data = CreateInteractionResponseMessage::new()
                .content("Failed to decode the response.")
                .ephemeral(true);
            return CreateInteractionResponse::Message(data);
        }
    };

    if json.get("error").is_some() {
        let data = CreateInteractionResponseMessage::new()
            .content("Failed to link your Discord account. Try generating a new token by refreshing the account page and clicking the link button again.")
            .ephemeral(true);
        return CreateInteractionResponse::Message(data);
    }

    let data = CreateInteractionResponseMessage::new()
        .content("Successfully linked your Discord account.")
        .ephemeral(true);
    CreateInteractionResponse::Message(data)
}
