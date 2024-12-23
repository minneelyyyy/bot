mod commands;

pub mod common;
pub mod inventory;
use crate::common::{Context, Error, Data};

use std::env;
use std::sync::Arc;

use poise::serenity_prelude::{self as serenity};
use poise::PartialContext;
use clap::Parser;

use sqlx::postgres::PgPoolOptions;
use sqlx::Row;

#[derive(Parser, Debug)]
struct BotArgs {
    /// Prefix for the bot (if unspecified, the bot will not have one)
    #[arg(short, long)]
    prefix: Option<String>,
}

async fn event_handler(
    _ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Message { new_message: message } => {
            if message.author.bot { return Ok(()) }
        }
        _ => (),
    }

    Ok(())
}

async fn get_prefix(ctx: PartialContext<'_, Data, Error>) -> Result<Option<String>, Error> {
    let guild = match ctx.guild_id {
        Some(guild) => guild,
        None => return Ok(None),
    };

    let db = &ctx.data.database;

    let prefix = sqlx::query("SELECT prefix FROM settings WHERE guildid = $1")
        .bind(guild.get() as i64)
        .fetch_one(db).await.ok()
            .map(|x| x.get(0)).unwrap_or(ctx.data.prefix.clone());

    Ok(prefix)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::dotenv().ok();
    let args = BotArgs::parse();

    let token = env::var("DISCORD_BOT_TOKEN")?;
    let database_url = env::var("DATABASE_URL")?;
    let intents = args.prefix.clone().map(|_|
        serenity::GatewayIntents::GUILD_MESSAGES
            | serenity::GatewayIntents::DIRECT_MESSAGES
            | serenity::GatewayIntents::MESSAGE_CONTENT)
        .unwrap_or(serenity::GatewayIntents::empty());

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::commands(),
            prefix_options: poise::PrefixFrameworkOptions {
                dynamic_prefix: Some(|ctx| Box::pin(get_prefix(ctx))),
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

                let database = PgPoolOptions::new()
                    .max_connections(5)
                    .connect(&database_url).await?;

                sqlx::query(
                    r#"
                    CREATE TABLE IF NOT EXISTS bank (
                        id BIGINT PRIMARY KEY,
                        balance INT
                    )
                    "#,
                ).execute(&database).await?;

                sqlx::query(
                    r#"
                    CREATE TABLE IF NOT EXISTS selfroles (
                        userid BIGINT NOT NULL,
                        guildid BIGINT NOT NULL,
                        roleid BIGINT,
                        UNIQUE (userid, guildid)
                    )
                    "#,
                ).execute(&database).await?;

                sqlx::query(
                    r#"
                    CREATE TABLE IF NOT EXISTS games (
                        id BIGSERIAL PRIMARY KEY,
                        name CHAR[255]
                    )
                    "#
                ).execute(&database).await?;

                sqlx::query(
                    r#"
                    CREATE TABLE IF NOT EXISTS items (
                        id BIGSERIAL PRIMARY KEY,
                        owner BIGINT NOT NULL,
                        game BIGINT NOT NULL,
                        item BIGINT NOT NULL,
                        data JSON NOT NULL,
                        name TEXT
                    )
                    "#
                ).execute(&database).await?;

                sqlx::query(
                    r#"
                    CREATE TABLE IF NOT EXISTS dailies (
                        userid BIGINT NOT NULL PRIMARY KEY,
                        last TIMESTAMPTZ,
                        streak INT
                    )
                    "#
                ).execute(&database).await?;

                sqlx::query(
                    r#"
                    CREATE TABLE IF NOT EXISTS settings (
                        guildid BIGINT NOT NULL PRIMARY KEY,
                        prefix TEXT
                    )
                    "#
                ).execute(&database).await?;

                println!("Bot is ready!");

                Ok(Data { database, prefix: args.prefix })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents).framework(framework).await;

    client.unwrap().start().await?;

    Ok(())
}
