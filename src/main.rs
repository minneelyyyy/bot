mod commands;
mod ping_limit;

pub mod common;
use crate::common::{Context, Error, Data};

use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::{Instant, Duration};

use poise::serenity_prelude as serenity;
use tokio::sync::Mutex;

use clap::Parser;

#[derive(Parser, Debug)]
struct BotArgs {
    /// Prefix for the bot (if unspecified, the bot will not have one)
    #[arg(short, long)]
    prefix: Option<String>,
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Message { new_message: message } => {
            let mentions = ping_limit::extract_mentions(&message.content);
            let mut cooldowns = data.mentions.lock().await;

            if mentions.iter()
                .filter(|&&id| id != message.author.id)
                .any(|mention| cooldowns.get(mention).map(|t| Instant::now().duration_since(*t) < Duration::from_secs(20)).unwrap_or(false))
            {
                message.reply(ctx, "stop spamming!").await?;

                let guild = match message.guild_id {
                    Some(g) => g,
                    None => return Ok(()),
                };

                let mut member = guild.member(ctx, message.author.id).await.unwrap();
                member.disable_communication_until_datetime(ctx,
                    serenity::Timestamp::from_unix_timestamp(serenity::Timestamp::now().unix_timestamp() + 60 as i64).unwrap()).await?;
            }

            for mention in mentions {
                cooldowns.insert(mention, Instant::now());
            }
        }
        _ => (),
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::dotenv().ok();
    let args = BotArgs::parse();

    let token = env::var("DISCORD_BOT_TOKEN")?;
    let intents = serenity::GatewayIntents::all();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::commands(),
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: args.prefix,
                edit_tracker: Some(Arc::new(
                    poise::EditTracker::for_timespan(std::time::Duration::from_secs(10)))),
                case_insensitive_commands: true,
                ..Default::default()
            },
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    users: Arc::new(Mutex::new(HashMap::new())),
                    mentions: Arc::new(Mutex::new(HashMap::new())),
                })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents).framework(framework).await;

    client.unwrap().start().await?;

    Ok(())
}
