use crate::common::{Context, Error};
use poise::serenity_prelude::UserId;
use sqlx::Row;

enum LeaderboardType {
    Tokens(usize),
    Dailies(usize),
}

async fn display_leaderboard(ctx: Context<'_>, t: LeaderboardType) -> Result<(), Error> {
    let db = &ctx.data().database;

    match t {
        LeaderboardType::Tokens(count) => {
            let rows = sqlx::query(
                r#"
                SELECT id, balance FROM bank
                ORDER BY balance DESC
                LIMIT $1
                "#,
            )
            .bind(count as i32)
            .fetch_all(db)
            .await?;

            let users: Vec<(_, i32)> = rows
                .iter()
                .map(|row| (UserId::new(row.get::<i64, _>(0) as u64), row.get(1)))
                .collect();
            let mut output = String::new();

            for (id, balance) in users {
                let user = id.to_user(ctx).await?;
                output += &format!("{} - {}\n", user.display_name(), balance);
            }

            ctx.reply(format!("```\n{output}```")).await?;
        }
        LeaderboardType::Dailies(count) => {
            let rows = sqlx::query(
                r#"
                SELECT userid, streak FROM dailies
                ORDER BY streak DESC
                LIMIT $1
                "#,
            )
            .bind(count as i32)
            .fetch_all(db)
            .await?;

            let users: Vec<(_, i32)> = rows
                .iter()
                .map(|row| (UserId::new(row.get::<i64, _>(0) as u64), row.get(1)))
                .collect();
            let mut output = String::new();

            for (id, streak) in users {
                let user = id.to_user(ctx).await?;
                output += &format!("{} - {}\n", user.display_name(), streak);
            }

            ctx.reply(format!("```\n{output}```")).await?;
        }
    }

    Ok(())
}

/// Display users with the top highest balances
#[poise::command(slash_command, prefix_command)]
pub async fn tokens(ctx: Context<'_>, count: Option<usize>) -> Result<(), Error> {
    let count = count.unwrap_or(10);

    if count < 1 || count > 20 {
        ctx.reply(format!("Sorry, I cannot display {count} entries."))
            .await?;
        return Ok(());
    }

    display_leaderboard(ctx, LeaderboardType::Tokens(count)).await
}

/// Display users with the top highest daily streaks
#[poise::command(slash_command, prefix_command)]
pub async fn dailies(ctx: Context<'_>, count: Option<usize>) -> Result<(), Error> {
    let count = count.unwrap_or(10);

    if count < 1 || count > 20 {
        ctx.reply(format!("Sorry, I cannot display {count} entries."))
            .await?;
        return Ok(());
    }

    display_leaderboard(ctx, LeaderboardType::Dailies(count)).await
}

/// Display a leaderboard
#[poise::command(
    slash_command,
    prefix_command,
    aliases("lb"),
    subcommands("tokens", "dailies")
)]
pub async fn leaderboard(ctx: Context<'_>) -> Result<(), Error> {
    display_leaderboard(ctx, LeaderboardType::Tokens(10)).await
}
