use crate::{Data, Error};
use poise::serenity_prelude as serenity;
use serenity::{Context, GuildId, Message, all::ReactionType, prelude::Mentionable};

/// Respond to greetings and mentions
pub async fn message_handler(ctx: &Context, data: &Data, msg: &Message, guild_id: GuildId) -> Result<(), Error> {
    let bot_id = ctx.cache.current_user().id;
    if msg.author.id == bot_id
        || msg.guild_id != Some(guild_id)
        || !msg.content.contains(&ctx.cache.current_user().mention().to_string())
    {
        return Ok(());
    }

    let content = msg.content.to_lowercase();
    let greetings = ["hi", "hello", "howdy", "bonjour", "ciao", "hej", "hola", "yo"];

    for greeting in greetings {
        if content.contains(greeting) {
            msg.channel_id.say(ctx, format!("{greeting} :)")).await?;
            return Ok(());
        }
    }

    if content.contains("cope") {
        let emoji_id = data.config.cope_nn_id;
        msg.react(
            ctx,
            match emoji_id {
                Some(id) => ReactionType::Custom {
                    animated: false,
                    id,
                    name: Some("cope".into()),
                },
                None => ReactionType::Unicode("👋".into()),
            },
        )
        .await?;
    } else {
        msg.react(ctx, ReactionType::Unicode("👋".into())).await?;
    }

    Ok(())
}
