use crate::common::{Context, Error};

/// Put forward an amount of tokens to either lose or earn
#[poise::command(slash_command, prefix_command)]
pub async fn wager(
    ctx: Context<'_>,
    #[min = 1]
    amount: i32) -> Result<(), Error>
{
    let data = ctx.data();
    let mut wealth = super::get_balance(ctx.author().id, &data).await?;

    if wealth < amount {
        ctx.reply(format!("You do not have enough tokens (**{}**) to wager this amount.",
                          wealth)).await?;
        return Ok(());
    }

    if rand::random() {
        wealth += amount;
        ctx.reply(format!("You just gained **{}** token(s)! You now have **{}**.",
                          amount, wealth)).await?;
    } else {
        wealth -= amount;
        ctx.reply(format!("You've lost **{}** token(s), you now have **{}**.",
                          amount, wealth)).await?;
    }

    super::change_balance(ctx.author().id, wealth, &data).await?;

    Ok(())
}