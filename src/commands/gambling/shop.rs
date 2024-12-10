use crate::common::{Context, Error};
use crate::inventory::Inventory;
use super::Item;
use once_cell::sync::Lazy;
use poise::serenity_prelude as serenity;
use std::collections::HashMap;

static ITEMS: Lazy<HashMap<&'static str, (i32, &Item)>> = Lazy::new(|| {
    HashMap::from([
        ("Pile of Dirt", (10, &super::items::DIRT)),
        ("Pile of Sand", (10, &super::items::SAND)),
    ])
});

async fn autocomplete_shop<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> impl Iterator<Item = serenity::AutocompleteChoice> + use<'a> {
    let db = &ctx.data().database;
    let balance = super::get_balance(ctx.author().id, db).await;

    ITEMS.values()
        .filter(move |(_, item)| item.name.contains(partial))
        .map(move |(cost, item)| {            
            let balance = balance.as_ref().unwrap_or(cost);

            serenity::AutocompleteChoice::new(
                if cost > balance {
                    format!("{} ({cost} tokens) - {} - Can't Afford", item.name, item.desc)
                } else {
                    format!("{} ({cost} tokens) - {}", item.name, item.desc)
                },
                item.name
            )
        })
}

#[poise::command(slash_command, prefix_command)]
pub async fn buy(ctx: Context<'_>,
    #[autocomplete = "autocomplete_shop"]
    item: String,
    #[min = 1]
    count: Option<i32>) -> Result<(), Error>
{
    let count = count.unwrap_or(1);

    if count < 1 {
        ctx.reply("Ok, did you REALLY expect me to fall for that for a third time? You've gotta find a new trick.").await?;
        return Ok(());
    }

    let mut tx = ctx.data().database.begin().await?;

    if let Some((price, &ref item)) = ITEMS.get(item.as_str()) {
        let author = ctx.author();
        let balance = super::get_balance(author.id, &mut *tx).await?;

        let total = *price * count;

        if total > balance {
            ctx.reply(format!("You could not afford the items ({count}x **{}** cost(s) **{total}** tokens)", item.name)).await?;
            return Ok(())
        }

        let inventory = Inventory::new(author.id, Some(super::ID));

        for _ in 0..count {
            inventory.give_item(&mut *tx, item.clone().inv_item()).await?;
        }

        super::change_balance(author.id, balance - total, &mut *tx).await?;

        ctx.reply(format!("You have purchased {count}x {}.", item.name)).await?;
        tx.commit().await?;
    } else {
        ctx.reply(format!("The item {item} is not available in this shop.")).await?;
    }

    Ok(())
}
