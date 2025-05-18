use std::{error, fmt};
use poise::ReplyHandle;
use sqlx::{Pool, Postgres};

pub struct Data {
    pub database: Pool<Postgres>,
    pub prefix: Option<String>,
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

#[derive(Debug, Clone, Copy)]
pub enum BigBirbError {
    GuildOnly,
}

impl fmt::Display for BigBirbError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::GuildOnly => "This command must be run inside of a guild.",
        };

        write!(f, "{s}")
    }
}

impl error::Error for BigBirbError {}
