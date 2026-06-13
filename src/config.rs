use crate::Error;
use poise::serenity_prelude as serenity;
use serde_with::DisplayFromStr;
use serenity::{ChannelId, EmojiId, GuildId};
use tracing::info;

#[allow(dead_code, clippy::unreadable_literal)]
pub mod constants {
    /// Purple theme color
    pub const EMBED_COLOR: u32 = 0x913de2;
    /// Red color for errors
    pub const ERROR_COLOR: u32 = 0xFF0000;
    /// Green color for success messages
    pub const SUCCESS_COLOR: u32 = 0x00FF00;
}

/// Config populated from environment variables
#[serde_with::serde_as]
#[derive(serde::Deserialize, Debug)]
pub struct Config {
    /// Discord bot token (required)
    pub discord_token: String,
    /// Base URL for the API (optional)
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub api_base: Option<url::Url>,
    /// Backend token for API authentication (optional)
    pub backend_token: Option<String>,
    /// Discord guild ID for guild-specific commands (optional)
    pub guild_id: Option<GuildId>,
    /// If true, register commands in the guild specified by `guild_id`. Otherwise, register globally.
    #[serde(default)]
    pub register_guild_commands: bool,
    /// Cope emoji ID (optional)
    pub cope_nn_id: Option<EmojiId>,
    /// Member count channel ID (optional)
    pub member_count_id: Option<ChannelId>,
    /// Download count channel ID (optional)
    pub download_count_id: Option<ChannelId>,
    /// `UptimeRobot` URL (optional)
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub uptime_url: Option<url::Url>,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, Error> {
        dotenvy::dotenv().ok();

        let config = envy::from_env::<Self>()?;

        config.log_feature_status();
        Ok(config)
    }

    /// Log the status of optional features based on what's actually configured
    fn log_feature_status(&self) {
        match &self.api_base {
            None => info!("API_BASE not set: backend integration disabled"),
            Some(u) => info!("API_BASE={u}: backend integration enabled"),
        }
        if self.backend_token.is_none() {
            info!("BACKEND_TOKEN not set: join/leave events will not be reported");
        }
        if self.guild_id.is_none() {
            info!("GUILD_ID not set: info channels and metrics disabled");
        }
        if self.cope_nn_id.is_none() {
            info!("COPE_NN_ID not set: defaulting to wave emoji");
        }
        if self.uptime_url.is_none() {
            info!("UPTIME_URL not set: uptime monitoring disabled");
        }
        info!(
            "Command registration: {}",
            if self.register_guild_commands {
                "guild"
            } else {
                "global"
            }
        );
    }
}
