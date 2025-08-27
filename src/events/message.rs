use crate::{config::CONFIG, Error};
use poise::serenity_prelude as serenity;
use serenity::{
    all::{GuildId, ReactionType}, prelude::Mentionable, Context,
    EmojiId,
    Message,
};

/// Respond to greetings and mentions
pub async fn message_handler(ctx: &Context, msg: &Message) -> Result<(), Error> {
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
