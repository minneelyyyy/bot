use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Generously donate your tokens to someone else
#[poise::command(slash_command, prefix_command)]
pub async fn give(ctx: Context<'_>, user: serenity::User, #[min = 1] amount: i32) -> Result<(), Error> {
    if user.bot {
        ctx.reply("Don't waste your tokens by giving them to a bot!").await?;
        return Ok(());
    }

    if amount < 1 {
        ctx.reply("You cannot give someone a negative amount of money (that's stealing!).").await?;
        return Ok(());
    }

    let mut tx = ctx.data().database.begin().await?;

    let author_balance = super::get_balance(ctx.author().id, &mut *tx).await?;

    if author_balance < amount {
        ctx.reply(format!("You do not have a high enough balance (**{author_balance}**) to complete this transaction.")).await?;
    } else {
        let author_new_balance = author_balance - amount;
        let reciever_new_balance = super::get_balance(user.id, &mut *tx).await? + amount;

        super::change_balance(user.id, reciever_new_balance, &mut *tx).await?;
        super::change_balance(ctx.author().id, author_new_balance, &mut *tx).await?;

        ctx.reply(format!("You've given **{}** **{}** tokens!", user.display_name(), amount)).await?;
    }

    tx.commit().await?;

    Ok(())
}