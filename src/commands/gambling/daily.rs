use crate::{Context, Error};

use std::time::{Duration, Instant};

fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let seconds = total_seconds % 60;
    let minutes = (total_seconds / 60) % 60;
    let hours = total_seconds / 3600;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

/// Redeem 50 daily tokens.
#[poise::command(slash_command, prefix_command)]
pub async fn daily(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let mut db = data.database.lock().await;
    let db = db.as_mut();

    let id = ctx.author().id;

    let mut dailies = data.dailies.lock().await;

    match dailies.get_mut(&id) {
        Some(daily) => {
            
            if daily.elapsed() >= Duration::from_secs(24 * 60 * 60) {
                *daily = Instant::now();
                super::add_balance(id, 50, db).await?;
                ctx.reply("Added **50** credits to your account!").await?;
            } else {
                let until_next_daily = Duration::from_secs(24 * 60 * 60) - daily.elapsed();
                ctx.reply(format!("Your next daily will be available in **{}**.", format_duration(until_next_daily))).await?;
            }
        },
        None => {
            dailies.insert(id.clone(), Instant::now());
            super::add_balance(id, 50, db).await?;
            ctx.reply("Added **50** credits to your account!").await?;
        }
    }

    Ok(())
}
