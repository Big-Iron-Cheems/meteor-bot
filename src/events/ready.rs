use crate::{Data, Error, config::Config};
use anyhow::Context as AnyhowContext;
use axum::{Router, extract::State, http::StatusCode, response::Response, routing::get};
use poise::serenity_prelude as serenity;
use serde_json::Value;
use serenity::{ActivityData, ChannelId, Context, EditChannel, GuildId};
use std::time::Duration;
use tokio::{net::TcpListener, time};
use tracing::{error, info, warn};
use url::Url;

static UPDATE_PERIOD: Duration = Duration::from_mins(6);
static UPTIME_INTERVAL: Duration = Duration::from_mins(1);

pub fn ready_handler(ctx: &Context, data: &Data) {
    ctx.set_activity(Some(ActivityData::playing("Meteor Client")));

    if let Some(uptime_url) = data.config.uptime_url.clone() {
        spawn_uptime_task(data.clone(), uptime_url);
    }

    if let (Some(guild_id), Some(member_count_id), Some(download_count_id)) = (
        data.config.guild_id,
        data.config.member_count_id,
        data.config.download_count_id,
    ) {
        spawn_info_channel_tasks(ctx, data, guild_id, member_count_id, download_count_id);
    }

    if let Some(guild_id) = data.config.guild_id {
        spawn_metrics_server(ctx, data, guild_id);
    }
}

/// Uptime pinger task
fn spawn_uptime_task(data: Data, uptime_url: Url) {
    tokio::spawn(async move {
        let mut interval = time::interval(UPTIME_INTERVAL);

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let latency_ms = compute_avg_latency(&data).await;

                    let url = if uptime_url.as_str().ends_with("ping=") {
                        format!("{uptime_url}{latency_ms}")
                    } else {
                        uptime_url.to_string()
                    };

                    if let Err(e) = data.http_client.get(url).send().await {
                        error!("Failed to send uptime request: {e}");
                    }
                }
                () = data.cancel.cancelled() => {
                    info!("Uptime task shutting down");
                    break;
                }
            }
        }
    });
}

/// Compute average latency across shards
async fn compute_avg_latency(data: &Data) -> u128 {
    let (sum, count) = data
        .shard_runners
        .lock()
        .await
        .values()
        .filter_map(|r| r.latency.map(|d| d.as_millis()))
        .fold((0u128, 0u128), |(s, c), l| (s + l, c + 1));
    sum.checked_div(count).map_or(0, |avg| avg)
}

/// Info channel tasks
fn spawn_info_channel_tasks(
    ctx: &Context,
    data: &Data,
    guild_id: GuildId,
    member_count_id: ChannelId,
    download_count_id: ChannelId,
) {
    spawn_channel_updater(
        ctx.clone(),
        download_count_id,
        ChannelRole::Downloads,
        {
            let data = data.clone();
            move || {
                let data = data.clone();
                async move { get_download_count(&data.http_client, &data.config).await.ok() }
            }
        },
        data.clone(),
    );

    spawn_channel_updater(
        ctx.clone(),
        member_count_id,
        ChannelRole::Members,
        {
            let ctx = ctx.clone();
            move || {
                let ctx = ctx.clone();
                async move { ctx.cache.guild(guild_id).map(|g| g.member_count) }
            }
        },
        data.clone(),
    );

    info!("Updating info channels every {} seconds", UPDATE_PERIOD.as_secs());
}

/// The semantic role of an info channel, used to derive its display label.
#[derive(Clone, Copy)]
enum ChannelRole {
    Members,
    Downloads,
}

impl ChannelRole {
    const fn label(self) -> &'static str {
        match self {
            Self::Members => "Members",
            Self::Downloads => "Downloads",
        }
    }
}

/// Spawn periodic updater task
fn spawn_channel_updater<F, Fut>(ctx: Context, channel_id: ChannelId, role: ChannelRole, mut get_count: F, data: Data)
where
    F: FnMut() -> Fut + Send + 'static,
    Fut: Future<Output = Option<u64>> + Send,
{
    tokio::spawn(async move {
        let mut interval = time::interval(UPDATE_PERIOD);

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    match get_count().await {
                        Some(count) => {
                            if let Err(e) = update_channel_name(&ctx, channel_id, role, count).await {
                                error!("Failed to update channel {}: {e}", channel_id.get());
                            }
                        }
                        None => {
                            warn!("Could not retrieve count for channel {}", channel_id.get());
                        }
                    }
                }
                () = data.cancel.cancelled() => {
                    info!("Info channel updater ({}) shutting down", channel_id.get());
                    break;
                }
            }
        }
    });
}

/// Update channel name
async fn update_channel_name(ctx: &Context, channel_id: ChannelId, role: ChannelRole, count: u64) -> Result<(), Error> {
    let new_name = format!("{}: {}", role.label(), format_long(count));

    if channel_id
        .to_channel(ctx)
        .await?
        .guild()
        .as_ref()
        .is_some_and(|c| c.name != new_name)
    {
        channel_id.edit(ctx, EditChannel::new().name(new_name)).await?;
    }

    Ok(())
}

/// Axum app state
#[derive(Clone)]
struct AppState {
    ctx: Context,
    data: Data,
}

/// Metrics server
fn spawn_metrics_server(ctx: &Context, data: &Data, guild_id: GuildId) {
    if ctx.cache.guild(guild_id).is_none() {
        return;
    }

    let app_state = AppState {
        ctx: ctx.clone(),
        data: data.clone(),
    };

    let app = Router::new()
        .route("/metrics", get(prometheus_metrics))
        .with_state(app_state);

    let cancel = data.cancel.clone();

    tokio::spawn(async move {
        let listener = match TcpListener::bind("0.0.0.0:9400").await {
            Ok(l) => l,
            Err(e) => {
                error!("Failed to bind metrics listener: {e}");
                return;
            }
        };
        info!("Providing metrics on :9400/metrics");
        if let Err(e) = axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                cancel.cancelled().await;
                info!("Metrics server shutting down");
            })
            .await
        {
            error!("Metrics server error: {e}");
        }
    });
}

/// Prometheus handler
async fn prometheus_metrics(State(state): State<AppState>) -> Result<Response<String>, StatusCode> {
    let guild_id = state.data.config.guild_id.ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    let member_count = state.ctx.cache.guild(guild_id).map_or(0, |g| g.member_count);
    let body = format!(
        "# HELP meteor_discord_users_total Total number of Discord users in our server\n\
         # TYPE meteor_discord_users_total gauge\n\
         meteor_discord_users_total {member_count}"
    );

    Response::builder()
        .header("Content-Type", "text/plain")
        .body(body)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Download count
async fn get_download_count(http_client: &reqwest::Client, config: &Config) -> Result<u64, Error> {
    let api_base = config.api_base.as_ref().context("API base URL not configured")?;
    let url = api_base.join("stats")?;
    let stats = http_client.get(url).send().await?.json::<Value>().await?;
    stats["downloads"].as_u64().context("Failed to parse downloads")
}

/// Format a large number with a k/m/b/t suffix and up to one decimal place
fn format_long(value: u64) -> String {
    const THRESHOLDS: &[(u64, &str)] = &[
        (1_000_000_000_000, "t"),
        (1_000_000_000, "b"),
        (1_000_000, "m"),
        (1_000, "k"),
    ];

    for &(threshold, suffix) in THRESHOLDS {
        if value >= threshold {
            let scaled = value / (threshold / 10);
            let integer = scaled / 10;
            let frac = scaled % 10;
            return if frac == 0 {
                format!("{integer}{suffix}")
            } else {
                format!("{integer}.{frac}{suffix}")
            };
        }
    }

    value.to_string()
}
