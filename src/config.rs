use dotenvy::dotenv;
use poise::serenity_prelude as serenity;
use serenity::all::{ChannelId, EmojiId, GuildId};
use std::{env, sync::LazyLock};

#[allow(dead_code)]
pub mod constants {
    /// Purple theme color
    pub const EMBED_COLOR: u32 = 0x913de2;
    /// Red color for errors
    pub const ERROR_COLOR: u32 = 0xFF0000;
    /// Green color for success messages
    pub const SUCCESS_COLOR: u32 = 0x00FF00;
}

/// Config populated from environment variables
pub struct Config {
    /// Discord bot token
    pub discord_token: String,
    /// Base URL for the API
    pub api_base: Option<String>,
    /// Backend token for API authentication
    pub backend_token: Option<String>,
    /// Discord guild ID for guild-specific commands
    pub guild_id: Option<GuildId>,
    /// Cope emoji ID
    pub cope_nn_id: Option<EmojiId>,
    /// Member count channel ID
    pub member_count_id: Option<ChannelId>,
    /// Download count channel ID
    pub download_count_id: Option<ChannelId>,
    /// UptimeRobot URL
    pub uptime_url: Option<String>,
}

impl Config {
    /// Ensure the retrieved variable is set
    fn get_required_var(key: &str) -> String {
        env::var(key)
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| {
                panic!("Environment variable '{key}' is required and cannot be empty")
            })
    }

    /// Parse an optional non-empty string env var
    fn get_optional_nonempty_var(key: &str) -> Option<String> {
        env::var(key).ok().and_then(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        dotenv().ok();

        let discord_token = Self::get_required_var("DISCORD_TOKEN");

        let api_base = Self::get_optional_nonempty_var("API_BASE").or_else(|| {
            println!("API base URL not set, backend integration will be disabled");
            None
        });

        let backend_token = Self::get_optional_nonempty_var("BACKEND_TOKEN").or_else(|| {
            println!("Backend token not set, user join/leave events will not be reported");
            None
        });

        let guild_id = env::var("GUILD_ID")
            .ok()
            .and_then(|s| s.parse().ok().map(GuildId::new))
            .or_else(|| {
                println!("Guild ID not configured, skipping info channel updates");
                None
            });

        let cope_nn_id = env::var("COPE_NN_ID")
            .ok()
            .and_then(|s| s.parse().ok().map(EmojiId::new))
            .or_else(|| {
                println!("Cope emoji ID not set, defaulting to wave emoji");
                None
            });

        let member_count_id = env::var("MEMBER_COUNT_ID")
            .ok()
            .and_then(|s| s.parse().ok().map(ChannelId::new))
            .or_else(|| {
                println!("Member count channel ID not set, info channels will not be updated");
                None
            });

        let download_count_id = env::var("DOWNLOAD_COUNT_ID")
            .ok()
            .and_then(|s| s.parse().ok().map(ChannelId::new))
            .or_else(|| {
                println!("Download count channel ID not set, info channels will not be updated");
                None
            });

        let uptime_url = Self::get_optional_nonempty_var("UPTIME_URL").or_else(|| {
            println!("Uptime URL not set, uptime monitoring will be disabled");
            None
        });

        Self {
            discord_token,
            api_base,
            backend_token,
            guild_id,
            cope_nn_id,
            member_count_id,
            download_count_id,
            uptime_url,
        }
    }
}

/// Global static configuration instance
pub static CONFIG: LazyLock<Config> = LazyLock::new(Config::from_env);
