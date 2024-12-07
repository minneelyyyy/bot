use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Generously donate your tokens to someone else
#[poise::command(slash_command, prefix_command)]
pub async fn give(ctx: Context<'_>, user: serenity::User, #[min = 1] amount: i32) -> Result<(), Error> {
    if user.bot {
        ctx.reply("Don't waste your tokens by giving them to a bot!").await?;
        return Ok(());
    }

    let data = ctx.data();
    let mut db = data.database.lock().await;
    let db = db.as_mut();

    let author_balance = super::get_balance(ctx.author().id, db).await?;

    if author_balance < amount {
        ctx.reply(format!("You do not have a high enough balance (**{author_balance}**) to complete this transaction.")).await?;
    } else {
        let author_new_balance = author_balance - amount;
        let reciever_new_balance = super::get_balance(user.id, db).await? + amount;

        super::change_balance(user.id, reciever_new_balance, db).await?;
        super::change_balance(ctx.author().id, author_new_balance, db).await?;

        ctx.reply(format!("You've given **{}** **{}** tokens!", user.display_name(), amount)).await?;
    }

    Ok(())
}