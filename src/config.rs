use dotenvy::dotenv;
use std::env;
use std::sync::LazyLock;

pub struct Config {
    pub discord_token: String,
    pub api_base: String,
    pub backend_token: String,
    pub application_id: String,
    pub guild_id: String,
    pub cope_nn_id: String,
    pub member_count_id: String,
    pub download_count_id: String,
    pub uptime_url: String,
}

impl Config {
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

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| Config::from_env());
