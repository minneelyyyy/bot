use crate::{Context, Error};

use poise::serenity_prelude::{User, UserId};
use sqlx::{
    PgConnection,
    types::chrono::{DateTime, TimeZone, Utc},
};

use std::time::Duration;

pub async fn get_streak(conn: &mut PgConnection, user: UserId) -> Result<Option<i32>, Error> {
    let result = sqlx::query!(
        "SELECT streak FROM dailies WHERE userid = $1",
        user.get() as i64
    )
    .fetch_optional(conn)
    .await?;

    Ok(result.map(|r| r.streak).unwrap_or(None))
}

pub async fn set_streak(conn: &mut PgConnection, user: UserId, streak: i32) -> Result<(), Error> {
    sqlx::query!(
        "INSERT INTO dailies (userid, streak) VALUES ($1, $2)
         ON CONFLICT (userid) DO UPDATE SET streak = EXCLUDED.streak",
        user.get() as i64,
        streak
    )
    .execute(conn)
    .await?;

    Ok(())
}

pub async fn get_last(
    conn: &mut PgConnection,
    user: UserId,
) -> Result<Option<DateTime<Utc>>, Error> {
    let result = sqlx::query!(
        "SELECT last FROM dailies WHERE userid = $1",
        user.get() as i64
    )
    .fetch_optional(conn)
    .await?;

    Ok(result.map(|r| r.last).unwrap_or(None))
}

pub async fn set_last(
    conn: &mut PgConnection,
    user: UserId,
    last: DateTime<Utc>,
) -> Result<(), Error> {
    sqlx::query!(
        "INSERT INTO dailies (userid, last) VALUES ($1, $2)
         ON CONFLICT (userid) DO UPDATE SET last = EXCLUDED.last",
        user.get() as i64,
        last
    )
    .execute(conn)
    .await?;

    Ok(())
}

/// Tells you what your current daily streak is
#[poise::command(slash_command, prefix_command)]
pub async fn streak(ctx: Context<'_>, user: Option<User>) -> Result<(), Error> {
    let mut tx = ctx.data().database.begin().await?;

    let (user, who) = match user {
        Some(user) => (user.id, format!("{} has", user.display_name())),
        None => (ctx.author().id, "You have".to_string()),
    };

    ctx.reply(format!(
        "{who} a daily streak of **{}**",
        get_streak(&mut tx, user).await?.unwrap_or(0)
    ))
    .await?;
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
                begin = "You have not redeemed your daily in time and your streak has been reset. "
                    .to_string();
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

        ctx.reply(format!(
            "{begin}**{payout}** tokens were added to your balance.{end}"
        ))
        .await?;
    } else {
        ctx.reply(format!(
            "Your next daily is not available! It will be available <t:{}:R>.",
            next_daily.timestamp()
        ))
        .await?;
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
