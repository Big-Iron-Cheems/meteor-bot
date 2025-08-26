use crate::config::CONFIG;
use serenity::async_trait;
use serenity::model::application::Interaction;
use serenity::model::gateway::Ready;
use serenity::prelude::GatewayIntents;
use serenity::prelude::{Context, EventHandler};
use serenity::Client;
use tokio::signal;

mod commands;
mod config;
mod constants;
mod events;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        commands::register_commands(&ctx).await;
        events::handle_ready(&ctx).await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            commands::handle_slash_command(&ctx, &command).await;
        }
    }

    /*async fn message(&self, ctx: Context, new_message: Message) {
        todo!()
    }

    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        todo!()
    }

    async fn guild_member_removal(
        &self,
        ctx: Context,
        guild_id: GuildId,
        user: User,
        member_data_if_available: Option<Member>,
    ) {
        todo!()
    }*/
}

#[tokio::main]
async fn main() {
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILD_MEMBERS;

    let mut client = Client::builder(&CONFIG.discord_token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    tokio::select! {
        result = client.start() => {
            if let Err(err) = result {
                println!("Client error: {err:?}");
            }
        }
        _ = signal::ctrl_c() => {
            println!("\nReceived Ctrl+C, shutting down gracefully...");
            client.shard_manager.shutdown_all().await;
        }
    }
}
