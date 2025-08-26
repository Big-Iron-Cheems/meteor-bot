use reqwest;
use serde_json;
use serenity::builder::{
    CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use serenity::model::application::InteractionResponseFlags;
use serenity::model::application::ResolvedOption;

pub fn register() -> CreateCommand {
    CreateCommand::new("dog").description("dawg")
}

pub async fn handle(_options: &[ResolvedOption<'_>]) -> CreateInteractionResponse {
    let api_url = "https://some-random-api.com/img/dog";
    let resp = match reqwest::get(api_url).await {
        Ok(r) => r,
        Err(_) => {
            let data = CreateInteractionResponseMessage::new()
                .content("Failed to fetch dog image")
                .flags(InteractionResponseFlags::EPHEMERAL);
            return CreateInteractionResponse::Message(data);
        }
    };

    let json: serde_json::Value = match resp.json().await {
        Ok(j) => j,
        Err(_) => {
            let data = CreateInteractionResponseMessage::new()
                .content("Failed to decode the response")
                .flags(InteractionResponseFlags::EPHEMERAL);
            return CreateInteractionResponse::Message(data);
        }
    };

    let url = json.get("link").and_then(|u| u.as_str());

    match url {
        Some(url) => {
            let data = CreateInteractionResponseMessage::new().content(url);
            CreateInteractionResponse::Message(data)
        }
        None => {
            let data = CreateInteractionResponseMessage::new()
                .content("Failed to parse the response")
                .flags(InteractionResponseFlags::EPHEMERAL);
            CreateInteractionResponse::Message(data)
        }
    }
}
