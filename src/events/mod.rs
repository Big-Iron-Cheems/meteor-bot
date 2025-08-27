use crate::config::CONFIG;
use crate::{Data, Error};
use axum::{extract::State, http::StatusCode, response::Response, routing::get, Router};
use poise::{serenity_prelude as serenity, FrameworkContext};
use serde_json::Value;
use serenity::{
    all::{ActivityData, FullEvent, Member, ReactionType, User}, builder::EditChannel, prelude::Mentionable, Channel, ChannelId, Context,
    EmojiId,
    GuildId,
    Message,
};
use std::{future::Future, time::Duration};
use tokio::{net::TcpListener, time};

static UPDATE_PERIOD: Duration = Duration::from_secs(6 * 60); // 6 minutes
static UPTIME_INTERVAL: Duration = Duration::from_secs(60); // 60 seconds

#[derive(Clone)]
struct AppState {
    ctx: Context,
}

pub async fn event_handler(
    ctx: &Context,
    event: &FullEvent,
    _framework: FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        FullEvent::Ready { data_about_bot, .. } => {
            println!("Logged in as {}", data_about_bot.user.name);

            let activity = ActivityData::playing("Meteor Client");
            ctx.set_activity(Some(activity));

            // Start background tasks
            log_err("uptime_handler", uptime_ready_handler(data)).await;
            log_err(
                "info_channel_handler",
                info_channel_ready_handler(ctx, data),
            )
            .await;
            log_err("metrics_handler", metrics_ready_handler(ctx)).await;
        }
        FullEvent::GuildMemberAddition { new_member } => {
            log_err("user_joined_handler", user_joined_handler(data, new_member)).await;
        }
        FullEvent::GuildMemberRemoval { user, .. } => {
            log_err("user_left_handler", user_left_handler(data, user)).await;
        }
        FullEvent::Message { new_message } => {
            log_err("hello_handler", hello_handler(ctx, new_message)).await;
        }
        _ => {
            println!("Received event: {:?}", event.snake_case_name());
        }
    }

    Ok(())
}

/// Log error from event handlers
async fn log_err<F, T>(label: &str, fut: F)
where
    F: Future<Output = Result<T, Error>>,
{
    if let Err(e) = fut.await {
        eprintln!("Error in {}: {}", label, e);
    }
}

/// Respond to greetings and mentions
async fn hello_handler(ctx: &Context, msg: &Message) -> Result<(), Error> {
    let bot_id = ctx.cache.current_user().id;
    if msg.author.id == bot_id {
        return Ok(());
    }

    let guild_id = CONFIG
        .guild_id
        .parse::<u64>()
        .ok()
        .map(GuildId::new)
        .ok_or("Invalid guild ID")?;
    if msg.guild_id != Some(guild_id)
        || !msg
            .content
            .contains(&format!("{}", ctx.cache.current_user().mention()))
    {
        return Ok(());
    }

    let content = msg.content.to_lowercase();
    let greetings = [
        "hi", "hello", "howdy", "bonjour", "ciao", "hej", "hola", "yo",
    ];

    for greeting in greetings {
        if content.contains(greeting) {
            msg.channel_id
                .say(&ctx.http, format!("{} :)", greeting))
                .await?;
            return Ok(());
        }
    }

    if content.contains("cope") && !CONFIG.cope_nn_id.is_empty() {
        if let Ok(cope_emoji_id) = CONFIG.cope_nn_id.parse::<u64>().map(EmojiId::new) {
            msg.react(
                &ctx.http,
                ReactionType::Custom {
                    animated: false,
                    id: cope_emoji_id,
                    name: Some("cope".into()),
                },
            )
            .await?;
        } else {
            msg.react(&ctx.http, ReactionType::Unicode("👋".into()))
                .await?;
        }
    } else {
        msg.react(&ctx.http, ReactionType::Unicode("👋".into()))
            .await?;
    }

    Ok(())
}

/// Notify backend of user join
async fn user_joined_handler(data: &Data, member: &Member) -> Result<(), Error> {
    if CONFIG.backend_token.is_empty() {
        return Ok(());
    }

    let url = format!(
        "{}/discord/userJoined?id={}",
        CONFIG.api_base, member.user.id
    );

    data.http_client
        .post(&url)
        .header("Authorization", &CONFIG.backend_token)
        .send()
        .await?;

    Ok(())
}

/// Notify backend of user leave
async fn user_left_handler(data: &Data, user: &User) -> Result<(), Error> {
    if CONFIG.backend_token.is_empty() {
        return Ok(());
    }

    let url = format!("{}/discord/userLeft?id={}", CONFIG.api_base, user.id);

    data.http_client
        .post(&url)
        .header("Authorization", &CONFIG.backend_token)
        .send()
        .await?;

    Ok(())
}

/// Start uptime pinger task for UptimeRobot
async fn uptime_ready_handler(data: &Data) -> Result<(), Error> {
    if CONFIG.uptime_url.is_empty() {
        println!("Uptime URL not set, uptime requests will not be made");
        return Ok(());
    }

    let http_client = data.http_client.clone();
    let shard_runners = data.shard_runners.clone();
    let uptime_url = &CONFIG.uptime_url;

    tokio::spawn(async move {
        let mut interval = time::interval(UPTIME_INTERVAL);

        loop {
            interval.tick().await;

            // Compute average latency across shards that have a value
            let latency_ms = {
                let runners_map = shard_runners.lock().await;
                let valid_latencies = runners_map
                    .values()
                    .filter_map(|r| r.latency)
                    .map(|d| d.as_millis())
                    .collect::<Vec<u128>>();

                if valid_latencies.is_empty() {
                    0 // no latency data yet
                } else {
                    valid_latencies.iter().sum::<u128>() / valid_latencies.len() as u128
                }
            };

            let url = if uptime_url.ends_with("ping=") {
                format!("{}{}", uptime_url, latency_ms)
            } else {
                uptime_url.clone()
            };

            if let Err(e) = http_client.get(&url).send().await {
                eprintln!("Failed to send uptime request: {}", e);
            }
        }
    });

    println!(
        "Sending uptime requests every {} seconds",
        UPTIME_INTERVAL.as_secs()
    );
    Ok(())
}

/// Start info channel updater tasks
async fn info_channel_ready_handler(ctx: &Context, data: &Data) -> Result<(), Error> {
    let guild_id = CONFIG
        .guild_id
        .parse::<u64>()
        .ok()
        .map(GuildId::new)
        .ok_or("Invalid guild ID")?;

    if CONFIG.member_count_id.is_empty() || CONFIG.download_count_id.is_empty() {
        println!(
            "Member count or download count channel IDs not set, info channels will not be updated"
        );
        return Ok(());
    }

    let member_count_id = CONFIG
        .member_count_id
        .parse::<u64>()
        .ok()
        .map(ChannelId::new)
        .ok_or("Invalid member count channel ID")?;

    let download_count_id = CONFIG
        .download_count_id
        .parse::<u64>()
        .ok()
        .map(ChannelId::new)
        .ok_or("Invalid download count channel ID")?;

    // Download count updater
    spawn_updater(ctx.clone(), download_count_id, {
        let http_client = data.http_client.clone();
        move || {
            let http_client = http_client.clone();
            async move { get_download_count(&http_client).await.ok() }
        }
    })
    .await;

    // Member count updater
    spawn_updater(ctx.clone(), member_count_id, {
        let ctx = ctx.clone();
        move || {
            let ctx = ctx.clone();
            async move { ctx.cache.guild(guild_id).map(|g| g.member_count as i64) }
        }
    })
    .await;

    println!(
        "Updating info channels every {} seconds",
        UPDATE_PERIOD.as_secs()
    );
    Ok(())
}

async fn spawn_updater<F, Fut>(ctx: Context, channel_id: ChannelId, mut get_count: F)
where
    F: FnMut() -> Fut + Send + 'static,
    Fut: Future<Output = Option<i64>> + Send,
{
    tokio::spawn(async move {
        let mut interval = time::interval(UPDATE_PERIOD);
        loop {
            interval.tick().await;

            if let Some(count) = get_count().await {
                if let Err(e) = update_channel_name(&ctx, channel_id, count).await {
                    eprintln!("Failed to update channel {:?}: {}", channel_id, e);
                }
            }
        }
    });
}

async fn update_channel_name(
    ctx: &Context,
    channel_id: ChannelId,
    count: i64,
) -> Result<(), Error> {
    let new_name = format!(
        "{}: {}",
        if channel_id.get() == CONFIG.member_count_id.parse::<u64>().unwrap() {
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

/// Start metrics server for Prometheus
async fn metrics_ready_handler(ctx: &Context) -> Result<(), Error> {
    let guild_id = CONFIG
        .guild_id
        .parse::<u64>()
        .ok()
        .map(GuildId::new)
        .ok_or("Invalid guild ID")?;

    // Check if guild exists
    if ctx.cache.guild(guild_id).is_none() {
        println!("Guild not found in cache, metrics server will not be started");
        return Ok(());
    }

    let app_state = AppState { ctx: ctx.clone() };

    let app = Router::new()
        .route("/metrics", get(prometheus_metrics))
        .with_state(app_state);

    let listener = TcpListener::bind("0.0.0.0:9400").await?;
    println!("Providing metrics on :9400/metrics");

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            eprintln!("Metrics server error: {}", e);
        }
    });

    Ok(())
}

async fn prometheus_metrics(State(state): State<AppState>) -> Result<Response<String>, StatusCode> {
    let guild_id = CONFIG
        .guild_id
        .parse::<u64>()
        .ok()
        .map(GuildId::new)
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

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

async fn get_download_count(http_client: &reqwest::Client) -> Result<i64, Error> {
    let response = http_client
        .get(format!("{}/stats", CONFIG.api_base))
        .send()
        .await?;

    let stats = response.json::<Value>().await?;

    let downloads = stats["downloads"]
        .as_f64()
        .ok_or("Failed to parse downloads as number")?;

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
