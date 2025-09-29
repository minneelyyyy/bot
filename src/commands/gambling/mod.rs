pub mod balance;
pub mod blackjack;
pub mod daily;
pub mod give;
pub mod leaderboard;
pub mod shop;
pub mod wager;

use crate::{
    common::{Context, Error},
    inventory::{self, Inventory},
};
use poise::serenity_prelude::{self as serenity, futures::StreamExt, UserId};
use sqlx::{PgExecutor, Row};
use std::collections::HashMap;

#[derive(Clone)]
pub enum Effect {
    Multiplier(f64),
    Chance(f64),
}

#[derive(Clone)]
pub struct Item {
    pub name: &'static str,
    pub desc: &'static str,
    pub effects: &'static [Effect],
    pub id: u64,
}

impl Item {
    pub fn inv_item(self) -> inventory::Item {
        inventory::Item {
            id: 0,
            name: self.name.to_string(),
            game: ID as i64,
            item: self.id as i64,
            data: serde_json::json!({}),
        }
    }
}

const ID: u64 = 440;

mod items {
    use super::{Effect, Item};

    pub const DIRT: Item = Item {
        name: "Pile of Dirt",
        desc: "Returns a 1.01x multiplier on all earnings",
        effects: &[Effect::Multiplier(1.01)],
        id: id::DIRT,
    };

    pub const SAND: Item = Item {
        name: "Pile of Sand",
        desc: "Increase your odds of winning by 1%",
        effects: &[Effect::Chance(0.51)],
        id: id::SAND,
    };

    mod id {
        pub const DIRT: u64 = 1;
        pub const SAND: u64 = 2;
    }

    pub fn get_item_by_id(id: u64) -> Option<&'static Item> {
        match id {
            id::DIRT => Some(&DIRT),
            id::SAND => Some(&SAND),
            _ => None,
        }
    }

    pub fn get_item_by_name(name: &str) -> Option<&'static Item> {
        match name {
            "Pile of Dirt" => Some(&DIRT),
            "Pile of Sand" => Some(&SAND),
            _ => None,
        }
    }
}

pub async fn get_balance<'a, E>(id: UserId, db: E) -> Result<i32, Error>
where
    E: PgExecutor<'a>,
{
    let row = sqlx::query("SELECT balance FROM bank WHERE id = $1")
        .bind(id.get() as i64)
        .fetch_one(db)
        .await
        .ok();

    let balance = if let Some(row) = row {
        row.try_get("balance")?
    } else {
        100
    };

    Ok(balance)
}

pub async fn change_balance<'a, E>(id: UserId, balance: i32, db: E) -> Result<(), Error>
where
    E: PgExecutor<'a>,
{
    sqlx::query("INSERT INTO bank (id, balance) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET balance = EXCLUDED.balance")
        .bind(id.get() as i64)
        .bind(balance)
        .execute(db).await?;

    Ok(())
}

async fn autocomplete_inventory<'a>(
    ctx: Context<'a>,
    partial: &'a str,
) -> impl Iterator<Item = serenity::AutocompleteChoice> + use<'a> {
    let db = &ctx.data().database;

    let inventory = Inventory::new(ctx.author().id, Some(ID))
        .items(db)
        .await
        .fold(HashMap::<i64, usize>::new(), |mut acc, item| async {
            let item = item.unwrap();
            let x = acc.get(&item.item);
            acc.insert(item.item, x.map(|x| x + 1).unwrap_or(1));
            acc
        })
        .await;

    inventory
        .into_iter()
        .map(|(id, count)| (items::get_item_by_id(id as u64).unwrap(), count))
        .filter(move |(item, _)| item.name.contains(partial))
        .map(|(item, count)| {
            serenity::AutocompleteChoice::new(
                format!("{} - {} ({count}x)", item.desc, item.name),
                item.name,
            )
        })
}
