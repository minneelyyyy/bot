
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

/// Remove a row concerning a user's self role from the database
pub async fn remove_user_role<'a, E>(user: UserId, guild: GuildId, db: E) -> Result<(), Error>
    where E: PgExecutor<'a>,
{
    sqlx::query("DELETE FROM selfroles WHERE userid = $1 AND guildid = $2")
        .bind(user.get() as i64)
        .bind(guild.get() as i64)
        .execute(db).await?;

    Ok(())
}

pub async fn remove_role<'a, E>(role: RoleId, guild: GuildId, db: E) -> Result<(), Error>
where E: PgExecutor<'a>
{
    sqlx::query("DELETE FROM selfroles WHERE roleid = $1 AND guildid = $2")
        .bind(role.get() as i64)
        .bind(guild.get() as i64)
        .execute(db).await?;

    Ok(())
}

/// Replace a user's custom role with a new one
pub async fn update_user_role<'a, E>(user: UserId, guild: GuildId, role: RoleId, db: E) -> Result<(), Error>
    where E: PgExecutor<'a>,
{
    sqlx::query("INSERT INTO selfroles (userid, guildid, roleid) VALUES($1, $2, $3) ON CONFLICT (userid, guildid) DO UPDATE SET roleid = EXCLUDED.roleid")
        .bind(user.get() as i64)
        .bind(guild.get() as i64)
        .bind(role.get() as i64)
        .execute(db).await?;

    Ok(())
}

/// Get a user's personal role id from the database
pub async fn get_user_role<'a, E>(user: UserId, guild: GuildId, db: E) -> Result<Option<RoleId>, Error>
    where E: PgExecutor<'a>,
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