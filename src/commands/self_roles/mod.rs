
use crate::common::{Context, Error};
use sqlx::{PgExecutor, Row};
use poise::serenity_prelude::{RoleId, UserId, GuildId};

mod register;
mod whois;
mod color;
mod name;
mod disown;
mod remove;

#[poise::command(
    prefix_command,
    slash_command,
    subcommands(
        "register::register",
        "whois::whois",
        "color::color",
        "name::name",
        "disown::disown",
        "remove::remove",
    )
)]
pub async fn role(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

pub async fn get_user_role<'a, E>(user: UserId, guild: GuildId, db: E) -> Result<Option<RoleId>, Error>
where
    E: PgExecutor<'a>,
{
    match sqlx::query("SELECT roleid FROM selfroles WHERE userid = $1 AND guildid = $2")
        .bind(user.get() as i64)
        .bind(guild.get() as i64)
        .fetch_one(db).await
    {
        Ok(row) => Ok(Some(RoleId::new(row.try_get::<i64, usize>(0)? as u64))),
        Err(sqlx::Error::RowNotFound) => Ok(None),
        Err(e) => return Err(Box::new(e)),
    }
}