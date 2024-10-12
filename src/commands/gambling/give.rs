use crate::{Context, Error};
use super::get_user_wealth_mut;
use poise::serenity_prelude as serenity;

#[poise::command(slash_command, prefix_command)]
pub async fn give(ctx: Context<'_>, user: serenity::User, amount: usize) -> Result<(), Error> {
    if user.bot {
        ctx.reply("Don't waste your token(s) by giving them to a bot!").await?;
        return Ok(());
    }

    let mut users = ctx.data().users.lock().await;
    let author_wealth = get_user_wealth_mut(&mut users, ctx.author().id);

    if *author_wealth < amount {
        ctx.reply(format!("You only have **{}** token(s) and cannot give away **{}**.",
                          *author_wealth, amount)).await?;
        return Ok(());
    }

    *author_wealth -= amount;

    let receiver_wealth = get_user_wealth_mut(&mut users, user.id);
    *receiver_wealth += amount;

    ctx.reply(format!("You've given **{}** **{}** token(s).", user.name, amount)).await?;

    Ok(())
}