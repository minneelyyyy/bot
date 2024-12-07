use crate::common::{Error, Data};

use poise::serenity_prelude::{self as serenity, Message, UserId};
use regex::Regex;

use std::time::{Instant, Duration};

pub fn extract_mentions(content: &str) -> Vec<UserId> {
    // Define the regex pattern for user mentions
    let re = Regex::new(r"<@(\d+)>").unwrap();

    // Find all matches and capture the IDs
    re.captures_iter(content)
        .filter_map(|cap| cap.get(1).map(|id| id.as_str().parse().unwrap()))
        .collect()
}

pub async fn ping_spam_yeller(ctx: &serenity::Context, message: &Message, data: &Data) -> Result<(), Error> {
    let mentions = extract_mentions(&message.content);
    let mut cooldowns = data.mentions.lock().await;

    if mentions.iter()
        .filter(|&&id| id != message.author.id)
        .any(|mention| cooldowns.get(mention).map(|t| Instant::now().duration_since(*t) < Duration::from_secs(20)).unwrap_or(false))
    {
        message.reply_ping(ctx, "stop spamming!").await?;

        let guild = match message.guild_id {
            Some(g) => g,
            None => return Ok(()),
        };

        let mut member = guild.member(ctx, message.author.id).await.unwrap();
        member.disable_communication_until_datetime(ctx,
            serenity::Timestamp::from_unix_timestamp(serenity::Timestamp::now().unix_timestamp() + 60i64).unwrap()).await?;
    }

    for mention in mentions {
        cooldowns.insert(mention, Instant::now());
    }

    Ok(())
}