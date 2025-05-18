
use crate::common::{Context, Error};
use sqlx::{PgConnection, Row};
use poise::serenity_prelude::{EditRole, GuildId, Permissions, RoleId, UserId};

mod whois;
pub mod color;
pub mod name;
mod disown;

mod admin;

#[poise::command(
    prefix_command,
    slash_command,
    subcommands(
        "name::name",
        "color::color",
        "disown::disown",
        "whois::whois",
    )
)]
pub async fn role(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    subcommands(
        "admin::name",
        "admin::color",
        "admin::remove",
        "admin::set",
        "admin::give",
        "whois::whois",
    ),
    required_permissions = "MANAGE_ROLES",
)]
pub async fn editrole(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Edit a user's personal role, creates it with some default values if it doesn't exist.
pub async fn edit_role(ctx: Context<'_>, user: UserId, guild: GuildId, edit: EditRole<'_>, db: &mut PgConnection) -> Result<RoleId, Error>
{
    if let Some(role) = get_user_role(user, guild, db).await? {
        guild.role(ctx, role).await?.edit(ctx, edit).await?;
        Ok(role)
    } else {
        create_role(ctx, user, guild, edit, db).await
    }
}

async fn create_role(
    ctx: Context<'_>,
    user: UserId,
    guild: GuildId,
    edit: EditRole<'_>,
    db: &mut PgConnection) -> Result<RoleId, Error>
{
    let def = EditRole::new()
        .name(user.to_user(ctx).await?.name)
        .permissions(Permissions::empty())
        .position({
            match crate::commands::settings::get_positional_role(ctx, guild).await? {
                Some(role) => guild.role(ctx, role).await?.position,
                None => 0u16,
            }
        })
        .hoist(crate::commands::settings::get_hoist_selfroles(ctx, guild).await?);

    let member = guild.member(ctx, user).await?;

    let mut role = guild.create_role(ctx, def).await?;
    role.edit(ctx, edit).await?;
    member.add_role(ctx, &role).await?;
    update_user_role(user, guild, role.id, db).await?;

    Ok(role.id)
}

/// Remove a row concerning a user's self role from the database
pub async fn remove_user_role(user: UserId, guild: GuildId, db: &mut PgConnection) -> Result<(), Error> {
    sqlx::query("DELETE FROM selfroles WHERE userid = $1 AND guildid = $2")
        .bind(user.get() as i64)
        .bind(guild.get() as i64)
        .execute(db).await?;

    Ok(())
}

pub async fn remove_role(role: RoleId, guild: GuildId, db: &mut PgConnection) -> Result<(), Error> {
    sqlx::query("DELETE FROM selfroles WHERE roleid = $1 AND guildid = $2")
        .bind(role.get() as i64)
        .bind(guild.get() as i64)
        .execute(db).await?;

    Ok(())
}

/// Replace a user's custom role with a new one
pub async fn update_user_role(user: UserId, guild: GuildId, role: RoleId, db: &mut PgConnection) -> Result<(), Error> {
    sqlx::query("INSERT INTO selfroles (userid, guildid, roleid) VALUES($1, $2, $3) ON CONFLICT (userid, guildid) DO UPDATE SET roleid = EXCLUDED.roleid")
        .bind(user.get() as i64)
        .bind(guild.get() as i64)
        .bind(role.get() as i64)
        .execute(db).await?;

    Ok(())
}

/// Get a user's personal role id from the database
pub async fn get_user_role(user: UserId, guild: GuildId, db: &mut PgConnection) -> Result<Option<RoleId>, Error> {
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

/// Get a user from the role id
pub async fn get_user_by_role(role: RoleId, guild: GuildId, db: &mut PgConnection) -> Result<Option<UserId>, Error> {
    match sqlx::query("SELECT userid FROM selfroles WHERE roleid = $1 AND guildid = $2")
        .bind(role.get() as i64)
        .bind(guild.get() as i64)
        .fetch_one(db).await
    {
        Ok(row) => Ok(Some(UserId::new(row.try_get::<i64, usize>(0)? as u64))),
        Err(sqlx::Error::RowNotFound) => Ok(None),
        Err(e) => return Err(Box::new(e)),
    }
}