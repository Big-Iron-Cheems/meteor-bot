use crate::config::CONFIG;
use crate::{Data, Error};
use poise::{serenity_prelude as serenity, FrameworkContext};
use serenity::{
    all::{ActivityData, FullEvent, Member, ReactionType, User}, prelude::Mentionable, Context, EmojiId,
    GuildId,
    Message,
};

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

    if content.contains("cope") {
        let cope_emoji_id = CONFIG
            .cope_nn_id
            .parse::<u64>()
            .ok()
            .map(EmojiId::new)
            .ok_or("Invalid emoji ID")?;

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

    Ok(())
}

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
