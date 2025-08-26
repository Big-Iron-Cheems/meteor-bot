use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::application::CommandInteraction;
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::env;

mod help;
mod link;
mod moderation;
mod silly;
mod stats;

pub async fn register_commands(ctx: &Context) {
    let commands = vec![
        help::faq::register(),
        help::installation::register(),
        help::logs::register(),
        help::old_version::register(),
        silly::capy::register(),
        silly::cat::register(),
        silly::dog::register(),
        silly::monkey::register(),
        silly::panda::register(),
        // Add more commands here
    ];

    let (scope, result) = match env::var("GUILD_ID")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map(GuildId::new)
    {
        Some(guild_id) => {
            println!("Registering guild slash commands...");
            ("guild", guild_id.set_commands(&ctx.http, commands).await)
        }
        None => {
            println!(
                "GUILD_ID not found or invalid, registering global commands (may take up to 1 hour)"
            );
            (
                "global",
                Command::set_global_commands(&ctx.http, commands).await,
            )
        }
    };

    match result {
        Ok(commands) => println!("Registered {} {scope} slash commands", commands.len()),
        Err(why) => println!("Error registering {scope} commands: {why:?}"),
    }
}

pub async fn handle_slash_command(ctx: &Context, command: &CommandInteraction) {
    let response = match command.data.name.as_str() {
        "faq" => help::faq::handle(&command.data.options()).await,
        "installation" => help::installation::handle(&command.data.options()).await,
        "logs" => help::logs::handle(&command.data.options()).await,
        "old-versions" => help::old_version::handle(&command.data.options()).await,
        "capybara" => silly::capy::handle(&command.data.options()).await,
        "cat" => silly::cat::handle(&command.data.options()).await,
        "dog" => silly::dog::handle(&command.data.options()).await,
        "monkey" => silly::monkey::handle(&command.data.options()).await,
        "panda" => silly::panda::handle(&command.data.options()).await,
        _ => {
            let data = CreateInteractionResponseMessage::new()
                .content("Command not implemented")
                .ephemeral(true);
            CreateInteractionResponse::Message(data)
        }
    };

    if let Err(why) = command.create_response(&ctx.http, response).await {
        println!("Cannot respond to slash command: {why}");
    }
}
