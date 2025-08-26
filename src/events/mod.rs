use crate::config::CONFIG;
use serenity::all::{
    ActivityData, ChannelId, EmojiId, GuildId, Member, Message, OnlineStatus, ReactionType, User,
};
use serenity::builder::EditChannel;
use serenity::prelude::{Context, Mentionable};
use tokio::time::Duration;

// Hello/greeting handler
pub async fn handle_message(ctx: &Context, message: &Message) {
    // Ignore self messages
    if message.author.id == ctx.cache.current_user().id {
        return;
    }

    // Check if message is from the configured guild and bot is mentioned
    let guild_id = match message.guild_id {
        Some(id) => id,
        None => return, // DM or other non-guild message
    };

    if guild_id.to_string() != CONFIG.guild_id {
        return;
    }

    let bot_mention = ctx.cache.current_user().mention().to_string();
    if !message.content.contains(&bot_mention) {
        return;
    }

    // Check for greetings
    let greetings = [
        "hi", "hello", "howdy", "bonjour", "ciao", "hej", "hola", "yo",
    ];
    let content_lower = message.content.to_lowercase();

    for greeting in greetings {
        if content_lower.contains(greeting) {
            let response = format!("{} :)", greeting);
            if let Err(e) = message.reply(&ctx.http, response).await {
                println!("Error sending greeting reply: {:?}", e);
            }
            return;
        }
    }

    let reaction_type = if content_lower.contains("cope") && !CONFIG.cope_nn_id.is_empty() {
        // Try to parse custom emoji ID
        if let Ok(emoji_id) = CONFIG.cope_nn_id.parse::<u64>() {
            ReactionType::Custom {
                animated: false,
                id: EmojiId::new(emoji_id),
                name: Some("cope".to_string()),
            }
        } else {
            ReactionType::Unicode("👋".to_string())
        }
    } else {
        ReactionType::Unicode("👋".to_string())
    };

    if let Err(e) = message.react(&ctx.http, reaction_type).await {
        println!("Error adding reaction: {:?}", e);
    }
}

// User joined handler
pub async fn handle_guild_member_addition(ctx: &Context, new_member: &Member) {
    if CONFIG.backend_token.is_empty() || CONFIG.api_base.is_empty() {
        println!("Backend token or API base not configured, skipping user joined event");
        return;
    }

    let url = format!(
        "{}/discord/userJoined?id={}",
        CONFIG.api_base, new_member.user.id
    );

    let client = reqwest::Client::new();
    if let Err(e) = client
        .post(&url)
        .header("Authorization", &CONFIG.backend_token)
        .send()
        .await
    {
        println!("Failed to send user joined request: {:?}", e);
    }
}

// User left handler
pub async fn handle_guild_member_removal(
    ctx: &Context,
    guild_id: GuildId,
    user: &User,
    member_data: Option<&Member>,
) {
    if CONFIG.backend_token.is_empty() || CONFIG.api_base.is_empty() {
        println!("Backend token or API base not configured, skipping user left event");
        return;
    }

    // Only handle events from the configured guild
    if guild_id.to_string() != CONFIG.guild_id {
        return;
    }

    let url = format!("{}/discord/userLeft?id={}", CONFIG.api_base, user.id);

    let client = reqwest::Client::new();
    if let Err(e) = client
        .post(&url)
        .header("Authorization", &CONFIG.backend_token)
        .send()
        .await
    {
        println!("Failed to send user left request: {:?}", e);
    }
}

// Bot ready handler - equivalent to your botStartHandler
pub async fn handle_ready(ctx: &Context) {
    // Set bot status
    let activity = ActivityData::playing("Meteor Client");
    let status = OnlineStatus::Online;

    ctx.set_presence(Some(activity), status);

    println!("✓ Bot status set to 'Playing Meteor Client'");

    // Start periodic tasks
    start_uptime_pinger(ctx).await;
    start_info_channel_updater(ctx).await;
}

// Uptime pinger
async fn start_uptime_pinger(ctx: &Context) {
    if CONFIG.uptime_url.is_empty() {
        println!("Uptime URL not configured, skipping uptime pinger");
        return;
    }

    let uptime_url = CONFIG.uptime_url.clone();
    let ctx_clone = ctx.clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));

        loop {
            interval.tick().await;

            let url = if uptime_url.ends_with("ping=") {
                // Add heartbeat latency if URL expects it
                format!("{}0", uptime_url) // Placeholder - serenity doesn't expose heartbeat latency easily
            } else {
                uptime_url.clone()
            };

            if let Err(e) = reqwest::get(&url).await {
                println!("Failed to send uptime ping: {:?}", e);
            }
        }
    });

    println!("✓ Uptime pinger started (60s interval)");
}

// Info channel updater
async fn start_info_channel_updater(ctx: &Context) {
    if CONFIG.member_count_id.is_empty() && CONFIG.download_count_id.is_empty() {
        println!("No info channel IDs configured, skipping info channel updater");
        return;
    }

    let ctx_clone = ctx.clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(6 * 60)); // 6 minutes

        loop {
            interval.tick().await;
            update_info_channels(&ctx_clone).await;
        }
    });

    println!("✓ Info channel updater started (6min interval)");
}

async fn update_info_channels(ctx: &Context) {
    // Update member count channel
    if !CONFIG.member_count_id.is_empty() && !CONFIG.guild_id.is_empty() {
        if let Ok(guild_id) = CONFIG.guild_id.parse::<u64>() {
            let guild_id = GuildId::new(guild_id);
            let member_count = ctx
                .cache
                .guild(guild_id)
                .map(|g| g.member_count)
                .unwrap_or(0);

            update_channel_name(ctx, &CONFIG.member_count_id, member_count as i64).await;
        }
    }

    // Update download count channel
    if !CONFIG.download_count_id.is_empty() && !CONFIG.api_base.is_empty() {
        if let Ok(downloads) = fetch_download_count().await {
            update_channel_name(ctx, &CONFIG.download_count_id, downloads).await;
        }
    }
}

async fn fetch_download_count() -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!("{}/stats", CONFIG.api_base);
    let response = reqwest::get(&url).await?;
    let stats: serde_json::Value = response.json().await?;

    let downloads = stats["downloads"]
        .as_f64()
        .ok_or("Invalid downloads field")?;

    Ok(downloads as i64)
}

async fn update_channel_name(ctx: &Context, channel_id: &str, count: i64) {
    if let Ok(id) = channel_id.parse::<u64>() {
        let channel_id = ChannelId::new(id);

        if let Ok(channel) = channel_id.to_channel(&ctx.http).await {
            if let Some(mut guild_channel) = channel.guild() {
                let current_name = &guild_channel.name;

                // Find the last colon to preserve the prefix
                if let Some(colon_pos) = current_name.rfind(':') {
                    let prefix = &current_name[..=colon_pos];
                    let new_name = format!("{} {}", prefix, format_number(count));

                    if let Err(e) = guild_channel
                        .edit(&ctx.http, EditChannel::new().name(&new_name))
                        .await
                    {
                        println!("Failed to update channel name: {:?}", e);
                    }
                }
            }
        }
    }
}

fn format_number(value: i64) -> String {
    if value < 1000 {
        return value.to_string();
    }

    let suffixes = ["k", "m", "b", "t"];
    let mut value = value as f64;
    let mut suffix_index = 0;

    while value >= 1000.0 && suffix_index < suffixes.len() - 1 {
        value /= 1000.0;
        suffix_index += 1;
    }

    if suffix_index == 0 {
        format!("{}", value as i64)
    } else {
        format!("{:.2}{}", value, suffixes[suffix_index - 1])
    }
}
