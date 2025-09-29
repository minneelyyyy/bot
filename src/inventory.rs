use crate::common::Error;

use poise::serenity_prelude::{futures::Stream, UserId};
use sqlx::PgExecutor;

#[derive(Clone, sqlx::FromRow, Debug, PartialEq, Eq)]
pub struct Item {
    pub id: i64,
    pub name: String,
    pub game: i64,
    pub item: i64,
    pub data: sqlx::types::JsonValue,
}

pub struct Inventory {
    user: UserId,
    game: Option<u64>,
}

impl Inventory {
    pub fn new(user: UserId, game: Option<u64>) -> Self {
        Self { user, game }
    }

    pub async fn give_item<'a, E>(&self, db: E, item: Item) -> Result<(), Error>
    where
        E: PgExecutor<'a>,
    {
        sqlx::query(
            r#"
            INSERT INTO items (owner, game, item, data, name)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(self.user.get() as i64)
        .bind(self.game.unwrap() as i64)
        .bind(item.item)
        .bind(item.data)
        .bind(item.name)
        .execute(db)
        .await?;

        Ok(())
    }

    pub async fn get_item_of_type<'a, E>(&self, db: E, item: u64) -> Result<Option<Item>, Error>
    where
        E: PgExecutor<'a>,
    {
        let x = sqlx::query_as(
            r#"
            SELECT id, name, game, item, data FROM items
            where item = $1 AND owner = $2
            "#,
        )
        .bind(item as i64)
        .bind(self.user.get() as i64)
        .fetch_one(db)
        .await
        .ok();

        Ok(x)
    }

    pub async fn get_item_with_name<'a, E>(&self, db: E, name: &str) -> Result<Option<Item>, Error>
    where
        E: PgExecutor<'a>,
    {
        let x = sqlx::query_as(
            r#"
            SELECT id, name, game, item, data FROM items
            where name = $1 AND user = $2
            "#,
        )
        .bind(name)
        .bind(self.user.get() as i64)
        .fetch_one(db)
        .await
        .ok();

        Ok(x)
    }

    pub async fn remove_item<'a, E>(&self, db: E, item: i64) -> Result<(), Error>
    where
        E: PgExecutor<'a>,
    {
        sqlx::query(
            r#"
            DELETE FROM items
            WHERE id = $1
            "#,
        )
        .bind(item)
        .execute(db)
        .await?;

        Ok(())
    }

    pub async fn items<'a, E>(
        &self,
        db: E,
    ) -> impl Stream<Item = Result<Item, sqlx::Error>> + use<'a, E>
    where
        E: PgExecutor<'a> + 'a,
    {
        match self.game {
            Some(game) => sqlx::query_as(
                r#"
                    SELECT id, name, game, item, data FROM items
                    WHERE owner = $1 AND game = $2
                    "#,
            )
            .bind(self.user.get() as i64)
            .bind(game as i64)
            .fetch(db),
            None => sqlx::query_as(
                r#"
                    SELECT id, name, game, item, data FROM items
                    WHERE owner = $1
                    "#,
            )
            .bind(self.user.get() as i64)
            .fetch(db),
        }
    }
}
