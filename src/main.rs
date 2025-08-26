use crate::commands::get_commands;
use crate::config::CONFIG;
use poise::{serenity_prelude as serenity, FrameworkError};
use serenity::{ClientBuilder, GatewayIntents, GuildId};
use tokio::signal;

mod commands;
mod config;
mod constants;
mod events;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
    /// Shared HTTP client
    pub http_client: reqwest::Client,
}

#[tokio::main]
async fn main() {
    let options = poise::FrameworkOptions {
        commands: get_commands(),
        on_error: |error: FrameworkError<'_, Data, Error>| {
            Box::pin(async move {
                match error {
                    FrameworkError::Setup { error, .. } => {
                        panic!("Failed to start bot: {:?}", error)
                    }
                    FrameworkError::Command { error, ctx, .. } => {
                        println!("Error in command `{}`: {:?}", ctx.command().name, error);
                    }
                    error => {
                        if let Err(e) = poise::builtins::on_error(error).await {
                            println!("Error while handling error: {}", e)
                        }
                    }
                }
            })
        },
        pre_command: |ctx| {
            Box::pin(async move {
                println!("Executing command {}...", ctx.command().qualified_name);
            })
        },
        post_command: |ctx| {
            Box::pin(async move {
                println!("Executed command {}!", ctx.command().qualified_name);
            })
        },
        event_handler: |_ctx, event, _framework, _data| {
            Box::pin(async move {
                println!(
                    "Got an event in event handler: {:?}",
                    event.snake_case_name()
                );
                Ok(())
            })
        },
        ..Default::default()
    };

    let data = Data {
        http_client: reqwest::Client::new(),
    };

    let framework = poise::Framework::builder()
        .options(options)
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                println!("Logged in as {}", _ready.user.name);

                // Register commands
                let num_commands = framework.options().commands.len();
                if let Some(guild_id) = CONFIG.guild_id.parse::<u64>().ok().map(GuildId::new) {
                    poise::builtins::register_in_guild(
                        ctx,
                        &framework.options().commands,
                        guild_id,
                    )
                    .await?;
                    println!("Registered {} guild slash commands", num_commands);
                } else {
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                    println!("Registered {} global slash commands", num_commands);
                }

                Ok(data)
            })
        })
        .build();

    let intents =
        GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILD_MEMBERS;
    let mut client = ClientBuilder::new(&CONFIG.discord_token, intents)
        .framework(framework)
        .await
        .expect("Failed to create client");

    tokio::select! {
        result = client.start() => {
            if let Err(err) = result {
                eprintln!("Client error: {:?}", err);
            }
        }
        _ = signal::ctrl_c() => {
            println!("\nReceived CTRL+C, shutting down gracefully...");
            client.shard_manager.shutdown_all().await;
        }
    }
}
