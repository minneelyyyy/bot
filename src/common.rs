use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use poise::serenity_prelude::UserId;
use sqlx::{Pool, Postgres};

pub struct Data {
    pub database: Pool<Postgres>,
    pub mentions: Arc<Mutex<HashMap<UserId, std::time::Instant>>>,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;