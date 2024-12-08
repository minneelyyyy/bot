mod commands;
mod ping_limit;

pub mod common;
use crate::common::{Context, Error, Data};

use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use poise::serenity_prelude::{self as serenity};
use tokio::sync::Mutex;

use clap::Parser;

use sqlx::{PgConnection, Connection};

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
            if message.author.bot { return Ok(()) }

            ping_limit::ping_spam_yeller(ctx, &message, data).await?;
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
    let database_url = env::var("DATABASE_URL")?;
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

                let mut database = PgConnection::connect(&database_url).await?;

                sqlx::query(
                    r#"
                    CREATE TABLE IF NOT EXISTS bank (
                        id BIGINT PRIMARY KEY,
                        balance INT
                    )
                    "#,
                ).execute(&mut database).await?;

                sqlx::query(
                    r#"
                    CREATE TABLE IF NOT EXISTS selfroles (
                        userid BIGINT NOT NULL,
                        guildid BIGINT NOT NULL,
                        roleid BIGINT,
                        UNIQUE (userid, guildid)
                    )
                    "#,
                ).execute(&mut database).await?;

                println!("Bot is ready!");

                Ok(Data {
                    database: Arc::new(Mutex::new(database)),
                    mentions: Arc::new(Mutex::new(HashMap::new())),
                    dailies: Arc::new(Mutex::new(HashMap::new())),
                })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents).framework(framework).await;

    client.unwrap().start().await?;

    Ok(())
}
