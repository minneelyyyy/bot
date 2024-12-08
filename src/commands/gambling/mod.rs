
pub mod balance;
pub mod give;
pub mod wager;
pub mod daily;

use crate::common::Error;
use poise::serenity_prelude::UserId;
use sqlx::{Row, PgConnection};

pub async fn get_balance(id: UserId, db: &mut PgConnection) -> Result<i32, Error> {
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

pub async fn change_balance(id: UserId, balance: i32, db: &mut PgConnection) -> Result<(), Error> {
    sqlx::query("INSERT INTO bank (id, balance) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET balance = EXCLUDED.balance")
        .bind(id.get() as i64)
        .bind(balance)
        .execute(db).await?;

    Ok(())
}

pub async fn add_balance(id: UserId, amount: i32, db: &mut PgConnection) -> Result<(), Error> {
    let balance = get_balance(id, db).await?;
    change_balance(id, balance + amount, db).await?;
    Ok(())
}