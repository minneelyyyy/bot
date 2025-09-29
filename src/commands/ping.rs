use crate::common::{Context, Error};
use poise::reply::CreateReply;
use std::time::Instant;

/// Display the bot's latency to Discord's REST and Gateway APIs
#[poise::command(slash_command, prefix_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let start = Instant::now();
    let msg = ctx.say("Pong! \u{1F3D3}").await?;
    let time = start.elapsed();

    msg.edit(
        ctx,
        CreateReply::default().content(format!(
            "Pong! \u{1F3D3}\nREST: {:.2?}\nGateway: {:.2?}",
            time,
            ctx.ping().await
        )),
    )
    .await?;

    Ok(())
}
