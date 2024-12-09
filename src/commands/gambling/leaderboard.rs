
use crate::common::{Context, Error};
use poise::serenity_prelude::UserId;
use sqlx::Row;

/// Display a leaderboard of the top 10 wealthiest players
#[poise::command(slash_command, prefix_command)]
pub async fn leaderboard(ctx: Context<'_>) -> Result<(), Error> {
    let db = &ctx.data().database;

    let rows = sqlx::query(
        r#"
        SELECT id, balance FROM bank
        ORDER BY balance DESC
        LIMIT 10
        "#
    ).fetch_all(db).await?;

    let users: Vec<(_, i32)> = rows.iter().map(|row| (UserId::new(row.get::<i64, _>(0) as u64), row.get(1))).collect();
    let mut output = String::new();

    for (id, balance) in users {
        let user = id.to_user(ctx).await?;
        output += &format!("{} - {}\n", user.display_name(), balance);
    }

    ctx.reply(format!("```\n{output}```")).await?;

    Ok(())
}
