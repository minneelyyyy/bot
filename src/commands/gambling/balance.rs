use crate::common::{Context, Error};
use poise::serenity_prelude as serenity;

/// Tells you what your or someone else's balance is
#[poise::command(slash_command, prefix_command)]
pub async fn balance(ctx: Context<'_>, user: Option<serenity::User>) -> Result<(), Error> {
    let user = user.as_ref().unwrap_or(ctx.author());

    let wealth = super::get_balance(user.id, &ctx.data()).await?;

    ctx.reply(format!("{} **{}** token(s).",
                      if user.id == ctx.author().id {
                          "You have".to_string()
                      } else {
                          format!("{} has", user.name)
                      }, wealth)).await?;

    Ok(())
}
