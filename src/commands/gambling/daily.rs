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
    let user = ctx.author().id;

    let day_ago = Instant::now() - Duration::from_secs(24 * 60 * 60);
    let last = *data.dailies.read().await.get(&user).unwrap_or(&day_ago);

    if last <= day_ago {
        data.dailies.write().await.insert(user, Instant::now());

        let db = &data.database;
        let mut tx = db.begin().await?;

        let bal = super::get_balance(user, &mut *tx).await?;
        super::change_balance(user, bal + 50, &mut *tx).await?;

        tx.commit().await?;

        ctx.reply(format!("**50** tokens have been added to your balance.")).await?;
    } else {
        let next = Duration::from_secs(24 * 60 * 60) - last.elapsed();
        ctx.reply(format!("Your next daily will be available in **{}**.", format_duration(next))).await?;
    }

    Ok(())
}
