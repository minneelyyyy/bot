
pub mod balance;
pub mod give;
pub mod wager;

use crate::common::{Data, Error};
use poise::serenity_prelude::UserId;
use sqlx::Row;

pub async fn get_balance(id: UserId, data: &Data) -> Result<i32, Error> {
    let db = &data.database;

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

pub async fn change_balance(id: UserId, balance: i32, data: &Data) -> Result<(), Error> {
    let db = &data.database;

    sqlx::query("INSERT INTO bank (id, balance) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET balance = EXCLUDED.balance")
        .bind(id.get() as i64)
        .bind(balance)
        .execute(db).await?;

    Ok(())
}