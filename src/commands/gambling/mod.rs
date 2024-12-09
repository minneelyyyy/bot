
pub mod balance;
pub mod give;
pub mod wager;
pub mod daily;
pub mod leaderboard;

use crate::common::Error;
use poise::serenity_prelude::UserId;
use sqlx::{Row, PgExecutor};

pub async fn get_balance<'a, E>(id: UserId, db: E) -> Result<i32, Error>
where
    E: PgExecutor<'a>,
{
    let row = sqlx::query("SELECT balance FROM bank WHERE id = $1")
        .bind(id.get() as i64)
        .fetch_one(db).await.ok();

    let balance = if let Some(row) = row {
        row.try_get("balance")?
    } else {
        100
    };

    Ok(balance)
}

pub async fn change_balance<'a, E>(id: UserId, balance: i32, db: E) -> Result<(), Error>
where
    E: PgExecutor<'a>,
{
    sqlx::query("INSERT INTO bank (id, balance) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET balance = EXCLUDED.balance")
        .bind(id.get() as i64)
        .bind(balance)
        .execute(db).await?;

    Ok(())
}
