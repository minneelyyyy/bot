use crate::common::{Context, Error};
use super::get_user_wealth_mut;

/// Put forward an amount of tokens to either lose or earn
#[poise::command(slash_command, prefix_command)]
pub async fn wager(ctx: Context<'_>, amount: usize) -> Result<(), Error> {
    let mut users = ctx.data().users.lock().await;

    let wealth = get_user_wealth_mut(&mut users, ctx.author().id);

    if *wealth < amount {
        ctx.reply(format!("You do not have enough tokens (**{}**) to wager this amount.",
                          *wealth)).await?;
        return Ok(());
    }

    if rand::random() {
        *wealth += amount;
        ctx.reply(format!("You just gained **{}** token(s)! You now have **{}**.",
                          amount, *wealth)).await?;
    } else {
        *wealth -= amount;
        ctx.reply(format!("You've lost **{}** token(s), you now have **{}**.",
                          amount, *wealth)).await?;
    }

    Ok(())
}