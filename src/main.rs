mod commands;

pub mod common;
use crate::common::{Context, Error, Data};

use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use poise::serenity_prelude as serenity;
use tokio::sync::Mutex;

use clap::Parser;

#[derive(Parser, Debug)]
struct BotArgs {
    /// Prefix for the bot (if unspecified, the bot will not have one)
    #[arg(short, long)]
    prefix: Option<String>,
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
                    poise::EditTracker::for_timespan(std::time::Duration::from_secs(3600)))),
                case_insensitive_commands: true,
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { users: Arc::new(Mutex::new(HashMap::new())) })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents).framework(framework).await;

    client.unwrap().start().await?;

    Ok(())
}
