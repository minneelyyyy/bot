use crate::common::{self, Context, Error};
use poise::serenity_prelude as serenity;

/// Tells you what your or someone else's balance is
#[poise::command(slash_command, prefix_command, aliases("bal", "b"))]
pub async fn balance(ctx: Context<'_>, user: Option<serenity::User>) -> Result<(), Error> {
    let user = user.as_ref().unwrap_or(ctx.author());
    let db = &ctx.data().database;

    let wealth = super::get_balance(user.id, db).await?;

    common::no_ping_reply(
        &ctx,
        format!(
            "{} **{}** token(s).",
            if user.id == ctx.author().id {
                "You have".to_string()
            } else {
                format!("{} has", user)
            },
            wealth
        ),
    )
    .await?;

    Ok(())
}
