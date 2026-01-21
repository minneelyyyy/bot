mod commands;

pub mod common;
pub mod inventory;
use crate::commands::self_roles;
use crate::common::{Context, Data, Error};

use std::env;
use std::sync::Arc;

use clap::Parser;
use poise::PartialContext;
use poise::serenity_prelude::{self as serenity};

use sqlx::Row;
use sqlx::postgres::PgPoolOptions;

#[derive(Parser, Debug)]
struct BotArgs {
    /// Prefix for the bot. If unspecified, the bot will not have one and will also not have access to message content.
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
        serenity::FullEvent::Message {
            new_message: message,
        } => {
            if message.author.bot {
                return Ok(());
            }
        }
        serenity::FullEvent::GuildMemberRemoval { guild_id, user, .. } => {
            let mut tx = data.database.begin().await?;

            if let Some(role) = self_roles::get_user_role(user.id, *guild_id, &mut *tx).await? {
                guild_id.delete_role(ctx, role).await?;
                self_roles::remove_role(role, *guild_id, &mut *tx).await?;
                tx.commit().await?;
            }
        }
        serenity::FullEvent::GuildRoleDelete {
            guild_id,
            removed_role_id,
            ..
        } => {
            let mut tx = data.database.begin().await?;

            self_roles::remove_role(*removed_role_id, *guild_id, &mut *tx).await?;

            tx.commit().await?;
        }
        _ => (),
    }

    Ok(())
}

async fn get_prefix(ctx: PartialContext<'_, Data, Error>) -> Result<Option<String>, Error> {
    let Some(guild) = ctx.guild_id else {
        return Ok(ctx.data.prefix.clone());
    };

    let db = &ctx.data.database;

    let prefix = match sqlx::query("SELECT prefix FROM settings WHERE guildid = $1")
        .bind(guild.get() as i64)
        .fetch_one(db)
        .await
    {
        Ok(r) => r.get(0),
        Err(sqlx::Error::RowNotFound) => None,
        Err(e) => return Err(Box::new(e)),
    };

    Ok(prefix.or(ctx.data.prefix.clone()))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::dotenv().ok();
    let args = BotArgs::parse();

    let token = env::var("DISCORD_BOT_TOKEN")?;
    let database_url = env::var("DATABASE_URL")?;
    let intents = args
        .prefix
        .clone()
        .map(|_| {
            serenity::GatewayIntents::GUILD_MESSAGES
                | serenity::GatewayIntents::DIRECT_MESSAGES
                | serenity::GatewayIntents::MESSAGE_CONTENT
        })
        .unwrap_or(serenity::GatewayIntents::empty())
        | serenity::GatewayIntents::GUILD_MEMBERS
        | serenity::GatewayIntents::GUILDS;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::commands(),
            prefix_options: poise::PrefixFrameworkOptions {
                dynamic_prefix: Some(|ctx| Box::pin(get_prefix(ctx))),
                edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                    std::time::Duration::from_secs(10),
                ))),
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
                    .connect(&database_url)
                    .await?;

                sqlx::migrate!().run(&database).await?;

                println!("Bot is ready!");

                Ok(Data {
                    database,
                    prefix: args.prefix,
                })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await?;

    Ok(())
}
