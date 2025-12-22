#![warn(clippy::all, clippy::pedantic)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use crate::config::Config;
use anyhow::Context;
use poise::{FrameworkError, serenity_prelude as serenity};
use serenity::{ClientBuilder, GatewayIntents, ShardId, ShardRunnerInfo};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, watch, watch::Receiver};
use tracing::{error, info};

mod commands;
mod config;
mod events;

type Error = anyhow::Error;
type Ctx<'a> = poise::Context<'a, Data, Error>;
type AppCtx<'a> = poise::ApplicationContext<'a, Data, Error>;

pub struct Data {
    /// Bot configuration
    pub config: Arc<Config>,
    /// Shared HTTP client
    pub http_client: reqwest::Client,
    /// Shard runners information
    pub shard_runners: Arc<Mutex<HashMap<ShardId, ShardRunnerInfo>>>,
    /// Shutdown signal for background tasks
    pub shutdown_rx: Receiver<bool>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt().compact().init();

    // Load configuration from environment
    let config = Arc::new(Config::from_env()?);

    // Channel to signal shutdown to background tasks
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let options = poise::FrameworkOptions {
        commands: commands::get_commands(),
        on_error: |error: FrameworkError<'_, Data, Error>| {
            Box::pin(async move {
                match error {
                    #[allow(clippy::panic)]
                    FrameworkError::Setup { error, .. } => {
                        error!(?error, "Failed to start bot");
                        panic!("Failed to start bot: {error:?}");
                    }
                    FrameworkError::Command { error, ctx, .. } => {
                        info!("Error in command `{}`: {error:?}", ctx.command().name);
                    }
                    error => {
                        if let Err(e) = poise::builtins::on_error(error).await {
                            info!("Error while handling error: {e}");
                        }
                    }
                }
            })
        },
        event_handler: |ctx, event, framework, data| Box::pin(events::event_handler(ctx, event, framework, data)),
        ..Default::default()
    };

    let config_for_setup = Arc::clone(&config);
    let framework = poise::Framework::builder()
        .options(options)
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                // Register commands
                let num_commands = framework.options().commands.len();
                if config_for_setup.register_guild_commands
                    && let Some(guild_id) = config_for_setup.guild_id
                {
                    poise::builtins::register_in_guild(ctx, &framework.options().commands, guild_id).await?;
                    info!("Registered {num_commands} guild slash commands");
                } else {
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                    info!("Registered {num_commands} global slash commands");
                }

                // Initialize shared data
                let data = Data {
                    config: config_for_setup,
                    http_client: reqwest::Client::new(),
                    shard_runners: framework.shard_manager().runners.clone(),
                    shutdown_rx,
                };

                Ok(data)
            })
        })
        .build();

    let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILD_MEMBERS;
    let mut client = ClientBuilder::new(&config.discord_token, intents)
        .framework(framework)
        .await
        .context("Failed to create Discord client")?;

    tokio::select! {
        result = client.start() => {
            result.context("Client error")?;
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Received CTRL+C, shutting down gracefully...");
            let _ = shutdown_tx.send(true);
            client.shard_manager.shutdown_all().await;
        }
    }

    Ok(())
}
