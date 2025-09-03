use crate::Error;
use poise::serenity_prelude as serenity;
use serde::Deserialize;
use serenity::all::{ChannelId, EmojiId, GuildId};
use tracing::info;

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
#[derive(Deserialize, Debug)]
pub struct Config {
    /// Discord bot token (required)
    pub discord_token: String,
    /// Base URL for the API (optional)
    pub api_base: Option<String>,
    /// Backend token for API authentication (optional)
    pub backend_token: Option<String>,
    /// Discord guild ID for guild-specific commands (optional)
    pub guild_id: Option<GuildId>,
    /// If true, register commands in the guild specified by guild_id. Otherwise, register globally.
    #[serde(default)]
    pub register_guild_commands: bool,
    /// Cope emoji ID (optional)
    pub cope_nn_id: Option<EmojiId>,
    /// Member count channel ID (optional)
    pub member_count_id: Option<ChannelId>,
    /// Download count channel ID (optional)
    pub download_count_id: Option<ChannelId>,
    /// UptimeRobot URL (optional)
    pub uptime_url: Option<String>,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, Error> {
        dotenvy::dotenv().ok();

        let config = envy::from_env::<Config>()?;

        config.log_feature_status();
        Ok(config)
    }

    /// Log the status of optional features based on what's actually configured
    fn log_feature_status(&self) {
        if self.api_base.is_none() {
            info!("API base URL not set, backend integration will be disabled");
        } else {
            info!("API base URL configured: backend integration enabled");
        }

        if self.backend_token.is_none() {
            info!("Backend token not set, user join/leave events will not be reported");
        } else {
            info!("Backend token configured: user join/leave events will be reported");
        }

        if self.guild_id.is_none() {
            info!("Guild ID not configured, skipping info channel updates");
        } else {
            info!("Guild ID configured: info channel updates enabled");
        }

        if self.cope_nn_id.is_none() {
            info!("Cope emoji ID not set, defaulting to wave emoji");
        } else {
            info!("Cope emoji ID configured");
        }

        if self.member_count_id.is_none() {
            info!("Member count channel ID not set, info channels will not be updated");
        } else {
            info!("Member count channel ID configured");
        }

        if self.download_count_id.is_none() {
            info!("Download count channel ID not set, info channels will not be updated");
        } else {
            info!("Download count channel ID configured");
        }

        if self.uptime_url.is_none() {
            info!("Uptime URL not set, uptime monitoring will be disabled");
        } else {
            info!("Uptime URL configured: uptime monitoring enabled");
        }

        info!(
            "Guild commands will be registered: {}",
            if self.register_guild_commands {
                "locally"
            } else {
                "globally"
            }
        );
    }
}
