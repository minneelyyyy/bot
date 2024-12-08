use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use poise::{serenity_prelude::UserId, ReplyHandle};
use sqlx::PgConnection;

pub struct Data {
    pub database: Arc<Mutex<PgConnection>>,
    pub mentions: Arc<Mutex<HashMap<UserId, std::time::Instant>>>,

    /// last time the user redeemed a daily
    pub dailies: Arc<Mutex<HashMap<UserId, std::time::Instant>>>,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

use poise::serenity_prelude::builder::CreateAllowedMentions;
use poise::CreateReply;

pub async fn no_ping_reply<'a>(ctx: &'a Context<'_>, text: impl Into<String>) -> Result<ReplyHandle<'a>, Error> {
    Ok(ctx.send(
        CreateReply::default()
            .content(text.into())
            .reply(true)
            .allowed_mentions(CreateAllowedMentions::new())
    ).await?)
}
