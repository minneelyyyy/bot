use crate::{Context, Error};

use poise::serenity_prelude::UserId;
use sqlx::{types::chrono::{DateTime, Utc, TimeZone}, PgExecutor, Row};

use std::time::Duration;

async fn get_streak<'a, E>(db: E, user: UserId) -> Result<Option<i32>, Error>
where
    E: PgExecutor<'a>,
{
    match sqlx::query(
            "SELECT streak FROM dailies WHERE userid = $1"
        ).bind(user.get() as i64).fetch_one(db).await
    {
        Ok(row) => Ok(Some(row.get(0))),
        Err(sqlx::Error::RowNotFound) => Ok(None),
        Err(e) => Err(Box::new(e)),
    }
}

async fn set_streak<'a, E>(db: E, user: UserId, streak: i32) -> Result<(), Error>
where
    E: PgExecutor<'a>,
{
    sqlx::query("INSERT INTO dailies (userid, streak) VALUES ($1, $2) ON CONFLICT (userid) DO UPDATE SET streak = EXCLUDED.streak")
        .bind(user.get() as i64)
        .bind(streak)
        .execute(db).await?;

    Ok(())
}

async fn get_last<'a, E>(db: E, user: UserId) -> Result<Option<DateTime<Utc>>, Error>
where
    E: PgExecutor<'a>,
{
    match sqlx::query(
            "SELECT last FROM dailies WHERE userid = $1"
        ).bind(user.get() as i64).fetch_one(db).await
    {
        Ok(row) => Ok(Some(row.get(0))),
        Err(sqlx::Error::RowNotFound) => Ok(None),
        Err(e) => Err(Box::new(e)),
    }
}

async fn set_last<'a, E>(db: E, user: UserId, last: DateTime<Utc>) -> Result<(), Error>
where
    E: PgExecutor<'a>,
{
    sqlx::query("INSERT INTO dailies (userid, last) VALUES ($1, $2) ON CONFLICT (userid) DO UPDATE SET last = EXCLUDED.last")
        .bind(user.get() as i64)
        .bind(last)
        .execute(db).await?;

    Ok(())
}

/// Tells you what your current daily streak is
#[poise::command(slash_command, prefix_command)]
pub async fn streak(ctx: Context<'_>) -> Result<(), Error> {
    let db = &ctx.data().database;

    ctx.reply(format!("You have a daily streak of **{}**", get_streak(db, ctx.author().id).await?.unwrap_or(0))).await?;
    Ok(())
}

async fn do_claim(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let user = ctx.author().id;
    let mut tx = data.database.begin().await?;

    let last = get_last(&mut *tx, user).await?;
    let existed = last.is_some();
    let last = last.unwrap_or(Utc.timestamp_opt(0, 0).unwrap());

    let now = Utc::now();
    let next_daily = last + Duration::from_secs(24 * 60 * 60);
    let time_to_redeem = next_daily + Duration::from_secs(24 * 60 * 60);

    if now > next_daily {
        let mut begin = "".to_string();
        let mut end = "".to_string();

        let streak = if now < time_to_redeem {
            let streak = get_streak(&mut *tx, user).await?.unwrap_or(0);

            if existed {
                begin = format!("You have a streak of **{streak}**! ");
            }

            streak
        } else {
            if existed {
                begin = "You have not redeemed your daily in time and your streak has been reset. ".to_string();
            }

            0
        };

        if !existed {
            end = " Keep redeeming your daily to build up a streak of up to 7 days!".to_string();
        }

        let payout = 50 + 10 * streak.min(7);

        let balance = super::get_balance(user, &mut *tx).await?;
        super::change_balance(user, balance + payout, &mut *tx).await?;

        set_streak(&mut *tx, user, streak + 1).await?;
        set_last(&mut *tx, user, now).await?;

        tx.commit().await?;

        ctx.reply(format!("{begin}**{payout}** tokens were added to your balance.{end}")).await?;
    } else {
        ctx.reply(format!("Your next daily is not available! It will be available <t:{}:R>.", next_daily.timestamp())).await?;
    }

    Ok(())
}

// Redeem daily tokens.
#[poise::command(slash_command, prefix_command)]
pub async fn claim(ctx: Context<'_>) -> Result<(), Error> {
    do_claim(ctx).await
}

/// Redeem daily tokens.
#[poise::command(slash_command, prefix_command, subcommands("streak", "claim"))]
pub async fn daily(ctx: Context<'_>) -> Result<(), Error> {
    do_claim(ctx).await
}
