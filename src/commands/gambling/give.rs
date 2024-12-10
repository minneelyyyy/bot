use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Generously donate your tokens to someone else
#[poise::command(slash_command, prefix_command, aliases("g"))]
pub async fn give(ctx: Context<'_>, user: serenity::User, amount: i32) -> Result<(), Error> {
    if user.bot {
        ctx.reply("Don't waste your tokens by giving them to a bot!").await?;
        return Ok(());
    }

    if amount < 1 {
        ctx.reply("You cannot give someone a negative amount of money (that's stealing!).").await?;
        return Ok(());
    }

    if user == *ctx.author() {
        ctx.reply("You cannot give yourself money!").await?;
        return Ok(());
    }

    let mut tx = ctx.data().database.begin().await?;
    let balance = super::get_balance(ctx.author().id, &mut *tx).await?;

    if balance < amount {
        ctx.reply(format!("You do not have a high enough balance (**{balance}**) to complete this transaction.")).await?;
    } else {
        super::change_balance(user.id, super::get_balance(user.id, &mut *tx).await? + amount, &mut *tx).await?;
        super::change_balance(ctx.author().id, balance - amount, &mut *tx).await?;
        tx.commit().await?;

        ctx.reply(format!("You've given **{}** **{}** tokens!", user.display_name(), amount)).await?;
    }

    Ok(())
}