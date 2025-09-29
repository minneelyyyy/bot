use crate::common::{Context, Error};

/// Tells you the current version of BigBirb
#[poise::command(slash_command, prefix_command)]
pub async fn version(ctx: Context<'_>) -> Result<(), Error> {
    ctx.reply(format!(
        "version {} ({}/{})",
        env!("CARGO_PKG_VERSION"),
        env!("GIT_BRANCH"),
        env!("GIT_HASH")
    ))
    .await?;

    Ok(())
}
