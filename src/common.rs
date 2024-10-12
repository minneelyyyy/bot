use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use poise::serenity_prelude::UserId;

pub struct Data {
    pub users: Arc<Mutex<HashMap<UserId, usize>>>
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;