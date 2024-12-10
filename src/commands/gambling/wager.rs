use crate::{common::{Context, Error}, inventory::Inventory};
use super::Effect;
use rand::Rng;

/// Put forward an amount of tokens to either lose or earn
#[poise::command(slash_command, prefix_command)]
pub async fn wager(
    ctx: Context<'_>,
    amount: i32,
    #[autocomplete = "super::autocomplete_inventory"]
    item: Option<String>) -> Result<(), Error>
{
    // #[min = 1] does not appear to work with prefix commands
    if amount < 1 {
        ctx.reply("you cannot wager less than 1 token.").await?;
        return Ok(());
    }

    let mut tx = ctx.data().database.begin().await?;
    let mut balance = super::get_balance(ctx.author().id, &mut *tx).await?;

    if balance < amount {
        ctx.reply(format!("You do not have enough tokens (**{balance}**) to wager this amount.")).await?;
        return Ok(());
    }

    let item = if let Some(item) = item {
        let inventory = Inventory::new(ctx.author().id, Some(super::ID));

        match super::items::get_item_by_name(&item) {
            Some(item) => {
                if let Some(item) = inventory.get_item_of_type(&mut *tx, item.id).await? {
                    inventory.remove_item(&mut *tx, item.id).await?;
                } else {
                    ctx.reply(format!("You do not have a(n) {} to use.", item.name)).await?;
                    return Ok(());
                }

                Some(item)
            }
            None => {
                ctx.reply("item {item} does not exist.").await?;
                return Ok(());
            }
        }
    } else {
        None
    };

    let (multiplier, chance) = item.map(|item| item.effects.iter()
        .fold((1.0, 0.5), |(m, c), effect| match effect {
            Effect::Multiplier(m) => (*m, c),
            Effect::Chance(c) => (m, *c),
        })
    ).unwrap_or((1.0, 0.5));

    if rand::thread_rng().gen_bool(chance) {
        let win = (amount as f64 * multiplier) as i32;
        balance += win;
        ctx.reply(format!("You just gained **{}** token(s)! You now have **{}**.",
                          win, balance)).await?;
    } else {
        balance -= amount;
        ctx.reply(format!("You've lost **{}** token(s), you now have **{}**.",
                          amount, balance)).await?;
    }

    super::change_balance(ctx.author().id, balance, &mut *tx).await?;

    tx.commit().await?;

    Ok(())
}