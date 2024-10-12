use crate::common::{Context, Error};
use std::iter;

/// Pardner
#[poise::command(slash_command, prefix_command)]
pub async fn yeehaw(ctx: Context<'_>,
                #[min = 1]
                width: Option<usize>,
                #[min = 1]
                height: Option<usize>) -> Result<(), Error>
{
    ctx.reply(iter::repeat("\u{1F920}".to_string().repeat(width.unwrap_or(1)))
        .take(height.unwrap_or(1))
        .collect::<Vec<_>>()
        .join("\n")).await?;

    Ok(())
}
