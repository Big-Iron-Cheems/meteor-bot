use crate::{Data, Error, config::Config, events::log_err};
use anyhow::Context as AnyhowContext;
use axum::{Router, extract::State, http::StatusCode, response::Response, routing::get};
use poise::serenity_prelude as serenity;
use serde_json::Value;
use serenity::{ActivityData, ChannelId, Context, EditChannel, GuildId};
use std::{sync::Arc, time::Duration};
use tokio::{net::TcpListener, sync::watch::Receiver, time};
use tracing::{error, info};
use url::Url;

static UPDATE_PERIOD: Duration = Duration::from_secs(6 * 60); // 6 minutes
static UPTIME_INTERVAL: Duration = Duration::from_secs(60); // 60 seconds

pub async fn ready_handler(ctx: &Context, data: &Data) -> Result<(), Error> {
    let activity = ActivityData::playing("Meteor Client");
    ctx.set_activity(Some(activity));

    if let Some(uptime_url) = data.config.uptime_url.as_ref() {
        uptime_ready_handler(data, uptime_url);
    }

    if let (Some(guild_id), Some(member_count_id), Some(download_count_id)) = (
        data.config.guild_id,
        data.config.member_count_id,
        data.config.download_count_id,
    ) {
        info_channel_ready_handler(ctx, data, guild_id, member_count_id, download_count_id);
    }

    if let Some(guild_id) = data.config.guild_id {
        log_err("metrics_handler", metrics_ready_handler(ctx, data, guild_id)).await;
    }

    Ok(())
}

/// Start uptime pinger task for `UptimeRobot`
fn uptime_ready_handler(data: &Data, uptime_url: &Url) {
    let http_client = data.http_client.clone();
    let shard_runners = data.shard_runners.clone();
    let mut shutdown_rx = data.shutdown_rx.clone();
    let uptime_url = uptime_url.clone();

    tokio::spawn(async move {
        let mut interval = time::interval(UPTIME_INTERVAL);

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    // Compute average latency across shards
                    let latency_ms = {
                        let (sum, count) = shard_runners.lock().await
                            .values()
                            .filter_map(|r| r.latency.map(|d| d.as_millis()))
                            .fold((0u128, 0u128), |(sum, count), latency| (sum + latency, count + 1));

                        if count == 0 {
                            0
                        } else {
                            sum / count
                        }
                    };

                    let url = if uptime_url.as_str().ends_with("ping=") {
                        format!("{uptime_url}{latency_ms}")
                    } else {
                        uptime_url.to_string()
                    };

                    if let Err(e) = http_client.get(url).send().await {
                        error!("Failed to send uptime request: {e}");
                    }
                }
                _ = shutdown_rx.changed() => {
                    info!("uptime_ready_handler shutting down...");
                    break;
                }
            }
        }
    });
}

/// Start info channel updater tasks
fn info_channel_ready_handler(
    ctx: &Context,
    data: &Data,
    guild_id: GuildId,
    member_count_id: ChannelId,
    download_count_id: ChannelId,
) {
    // Download count updater
    spawn_updater(
        ctx.clone(),
        download_count_id,
        {
            let config = data.config.clone();
            let http_client = data.http_client.clone();
            move || {
                let config = config.clone();
                let http_client = http_client.clone();
                async move { get_download_count(&http_client, &config).await.ok() }
            }
        },
        data.shutdown_rx.clone(),
        data.config.clone(),
    );

    // Member count updater
    spawn_updater(
        ctx.clone(),
        member_count_id,
        {
            let ctx = ctx.clone();
            move || {
                let ctx = ctx.clone();
                async move { ctx.cache.guild(guild_id).map(|g| g.member_count) }
            }
        },
        data.shutdown_rx.clone(),
        data.config.clone(),
    );

    info!("Updating info channels every {} seconds", UPDATE_PERIOD.as_secs());
}

/// Spawn a task to periodically update a channel name with a count
fn spawn_updater<F, Fut>(
    ctx: Context,
    channel_id: ChannelId,
    mut get_count: F,
    mut shutdown_rx: Receiver<bool>,
    config: Arc<Config>,
) where
    F: FnMut() -> Fut + Send + 'static,
    Fut: Future<Output = Option<u64>> + Send,
{
    tokio::spawn(async move {
        let mut interval = time::interval(UPDATE_PERIOD);

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Some(count) = get_count().await && let Err(e) = update_channel_name(&ctx, channel_id, count, &config).await {
                        error!("Failed to update channel {:?}: {e}", channel_id.get());
                    }
                }
                _ = shutdown_rx.changed() => {
                    info!("info_channel_ready_handler (ID: {:?}) shutting down...", channel_id.get());
                    break;
                }
            }
        }
    });
}

async fn update_channel_name(ctx: &Context, channel_id: ChannelId, count: u64, config: &Config) -> Result<(), Error> {
    let new_name = format!(
        "{}: {}",
        if Some(channel_id) == config.member_count_id {
            "Members"
        } else {
            "Downloads"
        },
        format_long(count)
    );

    if channel_id
        .to_channel(ctx)
        .await?
        .guild()
        .filter(|c| c.name != new_name)
        .is_some()
    {
        channel_id.edit(ctx, EditChannel::new().name(new_name)).await?;
    }

    Ok(())
}

/// Application state for Axum
#[derive(Clone)]
struct AppState {
    ctx: Context,
    config: Arc<Config>,
}

/// Start metrics server for Prometheus
async fn metrics_ready_handler(ctx: &Context, data: &Data, guild_id: GuildId) -> Result<(), Error> {
    if ctx.cache.guild(guild_id).is_none() {
        info!("Guild not found in cache, metrics server will not be started");
        return Ok(());
    }

    let app_state = AppState {
        ctx: ctx.clone(),
        config: data.config.clone(),
    };

    let app = Router::new()
        .route("/metrics", get(prometheus_metrics))
        .with_state(app_state);

    let listener = TcpListener::bind("0.0.0.0:9400").await?;
    info!("Providing metrics on :9400/metrics");

    let mut shutdown_rx = data.shutdown_rx.clone();
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.changed().await;
                info!("metrics_ready_handler shutting down...");
            })
            .await
        {
            error!("Metrics server error: {e}");
        }
    });

    Ok(())
}

async fn prometheus_metrics(State(state): State<AppState>) -> Result<Response<String>, StatusCode> {
    let guild_id = state.config.guild_id.ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let member_count = state.ctx.cache.guild(guild_id).map_or(0, |g| g.member_count);

    let response = format!(
        "# HELP meteor_discord_users_total Total number of Discord users in our server\n# TYPE meteor_discord_users_total gauge\nmeteor_discord_users_total {member_count}"
    );

    Response::builder()
        .header("Content-Type", "text/plain")
        .body(response)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn get_download_count(http_client: &reqwest::Client, config: &Config) -> Result<u64, Error> {
    let api_base = config.api_base.as_ref().context("API base URL not configured")?;
    let url = api_base
        .join("stats")
        .context("failed to join API base URL with stats path")?;
    let response = http_client
        .get(url)
        .send()
        .await
        .context("Failed to send request to fetch download stats")?;
    let stats = response
        .json::<Value>()
        .await
        .context("Failed to decode download stats response")?;
    let downloads = stats["downloads"]
        .as_u64()
        .context("Failed to parse downloads as number")?;
    Ok(downloads)
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
fn format_long(value: u64) -> String {
    const SUFFIXES: &[&str] = &["k", "m", "b", "t"];

    if value < 1000 {
        return value.to_string();
    }

    let value_f = value as f64;
    let exponent = ((value_f.log10() / 3.0).floor() as usize).min(SUFFIXES.len() - 1);
    let divisor = 1000_f64.powi(exponent as i32);
    let scaled = value_f / divisor;

    // trim trailing zeros
    let mut s = format!("{scaled:.2}");
    if s.ends_with("00") {
        s.truncate(s.len() - 3);
    } else if s.ends_with('0') {
        s.truncate(s.len() - 1);
    }

    format!("{}{}", s, SUFFIXES[exponent - 1])
}
