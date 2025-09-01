use crate::{config::Config, events::log_err, Data, Error};
use anyhow::Context as AnyhowContext;
use axum::{extract::State, http::StatusCode, response::Response, routing::get, Router};
use poise::serenity_prelude as serenity;
use serde_json::Value;
use serenity::{
    all::{ActivityData, ChannelId}, builder::EditChannel,
    Channel,
    Context,
};
use std::{sync::Arc, time::Duration};
use tokio::{net::TcpListener, sync::watch::Receiver, time};
use tracing::{error, info};

static UPDATE_PERIOD: Duration = Duration::from_secs(6 * 60); // 6 minutes
static UPTIME_INTERVAL: Duration = Duration::from_secs(60); // 60 seconds

pub async fn ready_handler(ctx: &Context, data: &Data) -> Result<(), Error> {
    let activity = ActivityData::playing("Meteor Client");
    ctx.set_activity(Some(activity));

    // Start background tasks
    log_err("uptime_handler", uptime_ready_handler(data)).await;
    log_err(
        "info_channel_handler",
        info_channel_ready_handler(ctx, data),
    )
    .await;
    log_err("metrics_handler", metrics_ready_handler(ctx, data)).await;

    Ok(())
}

/// Start uptime pinger task for UptimeRobot
async fn uptime_ready_handler(data: &Data) -> Result<(), Error> {
    let Some(uptime_url) = &data.config.uptime_url else {
        return Ok(());
    };

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

                    let url = if uptime_url.ends_with("ping=") {
                        format!("{}{}", uptime_url, latency_ms)
                    } else {
                        uptime_url.clone()
                    };

                    if let Err(e) = http_client.get(&url).send().await {
                        error!("Failed to send uptime request: {}", e);
                    }
                }
                _ = shutdown_rx.changed() => {
                    info!("uptime_ready_handler shutting down...");
                    break;
                }
            }
        }
    });

    Ok(())
}

/// Start info channel updater tasks
async fn info_channel_ready_handler(ctx: &Context, data: &Data) -> Result<(), Error> {
    let Some(guild_id) = data.config.guild_id else {
        return Ok(());
    };

    let Some(member_count_id) = data.config.member_count_id else {
        return Ok(());
    };

    let Some(download_count_id) = data.config.download_count_id else {
        return Ok(());
    };

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
    )
    .await;

    // Member count updater
    spawn_updater(
        ctx.clone(),
        member_count_id,
        {
            let ctx = ctx.clone();
            move || {
                let ctx = ctx.clone();
                async move { ctx.cache.guild(guild_id).map(|g| g.member_count as i64) }
            }
        },
        data.shutdown_rx.clone(),
        data.config.clone(),
    )
    .await;

    info!(
        "Updating info channels every {} seconds",
        UPDATE_PERIOD.as_secs()
    );
    Ok(())
}

/// Spawn a task to periodically update a channel name with a count
async fn spawn_updater<F, Fut>(
    ctx: Context,
    channel_id: ChannelId,
    mut get_count: F,
    mut shutdown_rx: Receiver<bool>,
    config: Arc<Config>,
) where
    F: FnMut() -> Fut + Send + 'static,
    Fut: Future<Output = Option<i64>> + Send,
{
    tokio::spawn(async move {
        let mut interval = time::interval(UPDATE_PERIOD);

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Some(count) = get_count().await {
                        if let Err(e) = update_channel_name(&ctx, channel_id, count, &config).await {
                            error!("Failed to update channel {:?}: {}", channel_id.get(), e);
                        }
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

async fn update_channel_name(
    ctx: &Context,
    channel_id: ChannelId,
    count: i64,
    config: &Config,
) -> Result<(), Error> {
    let new_name = format!(
        "{}: {}",
        if Some(channel_id) == config.member_count_id {
            "Members"
        } else {
            "Downloads"
        },
        format_long(count)
    );

    let channel = channel_id.to_channel(&ctx.http).await?;
    if let Channel::Guild(channel) = channel {
        if channel.name != new_name {
            channel_id
                .edit(&ctx.http, EditChannel::new().name(new_name))
                .await?;
        }
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
async fn metrics_ready_handler(ctx: &Context, data: &Data) -> Result<(), Error> {
    let Some(guild_id) = data.config.guild_id else {
        return Ok(());
    };

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
            error!("Metrics server error: {}", e);
        }
    });

    Ok(())
}

async fn prometheus_metrics(State(state): State<AppState>) -> Result<Response<String>, StatusCode> {
    let Some(guild_id) = state.config.guild_id else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let member_count = state
        .ctx
        .cache
        .guild(guild_id)
        .map(|g| g.member_count)
        .unwrap_or(0);

    let response = format!(
        "# HELP meteor_discord_users_total Total number of Discord users in our server\n# TYPE meteor_discord_users_total gauge\nmeteor_discord_users_total {}",
        member_count
    );

    Ok(Response::builder()
        .header("Content-Type", "text/plain")
        .body(response)
        .unwrap())
}

async fn get_download_count(http_client: &reqwest::Client, config: &Config) -> Result<i64, Error> {
    let api_base = config
        .api_base
        .as_ref()
        .context("API base URL not configured")?;
    let response = http_client
        .get(format!("{}/stats", api_base))
        .send()
        .await
        .context("Failed to send request to fetch download stats")?;
    let stats = response
        .json::<Value>()
        .await
        .context("Failed to decode download stats response")?;
    let downloads = stats["downloads"]
        .as_f64()
        .context("Failed to parse downloads as number")?;
    Ok(downloads as i64)
}

fn format_long(value: i64) -> String {
    if value < 1000 {
        return value.to_string();
    }

    let suffixes = ["k", "m", "b", "t"];
    let exponent = ((value as f64).log10() / 3.0).floor() as usize;
    let exponent = exponent.min(suffixes.len());

    if exponent == 0 {
        return value.to_string();
    }

    let base = 1000_i64.pow(exponent as u32) as f64;
    let first = value as f64 / base;

    format!("{:.2}{}", first, suffixes[exponent - 1])
}
