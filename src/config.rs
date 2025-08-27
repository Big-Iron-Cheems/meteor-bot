use dotenvy::dotenv;
use std::env;
use std::sync::LazyLock;

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
    /// Discord bot token (required)
    pub discord_token: String,
    /// Base URL for the API
    pub api_base: String,
    /// Backend token for API authentication
    pub backend_token: String,
    /// Discord application ID
    pub application_id: String,
    /// Discord guild ID for guild-specific commands
    pub guild_id: String,
    /// Cope emoji ID
    pub cope_nn_id: String,
    /// Member count channel ID
    pub member_count_id: String,
    /// Download count channel ID
    pub download_count_id: String,
    /// UptimeRobot URL
    pub uptime_url: String,
}

impl Config {
    /// Ensure the retrieved variable is set
    fn get_required_var(key: &str) -> String {
        let value =
            env::var(key).unwrap_or_else(|_| panic!("Environment variable '{key}' is required"));

        let trimmed = value.trim();
        if trimmed.is_empty() {
            panic!(
                "Environment variable '{}' cannot be empty or whitespace-only",
                key
            );
        }

        trimmed.to_string()
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        dotenv().ok();

        Self {
            discord_token: Self::get_required_var("DISCORD_TOKEN"),
            api_base: env::var("API_BASE").unwrap_or_default(),
            backend_token: env::var("BACKEND_TOKEN").unwrap_or_default(),
            application_id: env::var("APPLICATION_ID").unwrap_or_default(),
            guild_id: env::var("GUILD_ID").unwrap_or_default(),
            cope_nn_id: env::var("COPE_NN_ID").unwrap_or_default(),
            member_count_id: env::var("MEMBER_COUNT_ID").unwrap_or_default(),
            download_count_id: env::var("DOWNLOAD_COUNT_ID").unwrap_or_default(),
            uptime_url: env::var("UPTIME_URL").unwrap_or_default(),
        }
    }
}

/// Global static configuration instance
pub static CONFIG: LazyLock<Config> = LazyLock::new(Config::from_env);
