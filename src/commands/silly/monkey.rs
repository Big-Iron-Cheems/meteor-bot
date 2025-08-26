use rand::Rng;
use serenity::builder::{
    CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use serenity::model::application::ResolvedOption;

pub fn register() -> CreateCommand {
    CreateCommand::new("monkey").description("monke")
}

pub async fn handle(_options: &[ResolvedOption<'_>]) -> CreateInteractionResponse {
    let mut rng = rand::rng();
    let w = rng.random_range(200..=1000);
    let h = rng.random_range(200..=1000);
    let url = format!("https://www.placemonkeys.com/{}/{}?random", w, h);
    let data = CreateInteractionResponseMessage::new().content(url);
    CreateInteractionResponse::Message(data)
}
