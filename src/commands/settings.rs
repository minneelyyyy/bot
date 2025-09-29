use crate::common::{self, BigBirbError, Context, Error};

use poise::serenity_prelude::{GuildId, Role, RoleId};
use sqlx::Row;

async fn get_prefix(ctx: Context<'_>, guild: GuildId) -> Result<Option<String>, Error> {
    let db = &ctx.data().database;

    let prefix: Option<String> = match sqlx::query("SELECT prefix FROM settings WHERE guildid = $1")
        .bind(guild.get() as i64)
        .fetch_one(db)
        .await
    {
        Ok(r) => r.get(0),
        Err(sqlx::Error::RowNotFound) => None,
        Err(e) => return Err(Box::new(e)),
    };

    Ok(prefix.or(ctx.data().prefix.clone()))
}

#[poise::command(prefix_command, slash_command)]
async fn prefix(ctx: Context<'_>, prefix: Option<String>) -> Result<(), Error> {
    let guild = ctx.guild_id().ok_or(BigBirbError::GuildOnly)?;

    match prefix {
        Some(prefix) => {
            let member = ctx.author_member().await.unwrap();

            if !member.permissions(ctx).iter().any(|p| p.manage_guild()) {
                ctx.reply("You do not have permission to change this setting.")
                    .await?;
                return Ok(());
            }

            let mut tx = ctx.data().database.begin().await?;

            sqlx::query("INSERT INTO settings (guildid, prefix) VALUES ($1, $2) ON CONFLICT (guildid) DO UPDATE SET prefix = EXCLUDED.prefix")
                .bind(guild.get() as i64)
                .bind(&prefix)
                .execute(&mut *tx).await?;

            tx.commit().await?;

            ctx.reply(format!(
                "This server's custom prefix has been updated to `{prefix}`."
            ))
            .await?;
        }
        None => {
            let s = get_prefix(ctx, guild)
                .await?
                .map(|s| format!("`{s}`"))
                .unwrap_or("not set".into());
            ctx.reply(format!("This server's command prefix is {s}."))
                .await?;
        }
    }

    Ok(())
}

pub async fn get_positional_role(
    ctx: Context<'_>,
    guild: GuildId,
) -> Result<Option<RoleId>, Error> {
    let db = &ctx.data().database;

    let role: Option<i64> =
        match sqlx::query("SELECT positional_role FROM settings WHERE guildid = $1")
            .bind(guild.get() as i64)
            .fetch_one(db)
            .await
        {
            Ok(r) => r.get(0),
            Err(sqlx::Error::RowNotFound) => None,
            Err(e) => return Err(Box::new(e)),
        };

    Ok(role.map(|sf| RoleId::new(sf as u64)))
}

#[poise::command(prefix_command, slash_command)]
pub async fn position(ctx: Context<'_>, role: Option<Role>) -> Result<(), Error> {
    let guild = ctx.guild_id().ok_or(BigBirbError::GuildOnly)?;
    let member = ctx.author_member().await.unwrap();

    if !member.permissions(ctx).iter().any(|p| p.manage_guild()) {
        ctx.reply("You do not have permission to see or change this setting.")
            .await?;
        return Ok(());
    }

    match role {
        Some(role) => {
            let mut tx = ctx.data().database.begin().await?;

            sqlx::query("INSERT INTO settings (guildid, positional_role) VALUES ($1, $2) ON CONFLICT (guildid) DO UPDATE SET positional_role = EXCLUDED.positional_role")
                .bind(guild.get() as i64)
                .bind(role.id.get() as i64)
                .execute(&mut *tx).await?;

            tx.commit().await?;

            common::no_ping_reply(
                &ctx,
                format!("The bot will now place newly created self roles below {role}."),
            )
            .await?;
        }
        None => {
            let s = match get_positional_role(ctx, guild).await? {
                Some(r) => format!("{}", guild.role(ctx, r).await?),
                None => "not set".into(),
            };

            ctx.reply(format!("This server's positional role is {s}."))
                .await?;
        }
    }

    Ok(())
}

pub async fn get_hoist_selfroles(ctx: Context<'_>, guild: GuildId) -> Result<bool, Error> {
    let db = &ctx.data().database;

    let hoist: Option<bool> =
        match sqlx::query("SELECT hoist_selfroles FROM settings WHERE guildid = $1")
            .bind(guild.get() as i64)
            .fetch_one(db)
            .await
        {
            Ok(r) => r.get(0),
            Err(sqlx::Error::RowNotFound) => None,
            Err(e) => return Err(Box::new(e)),
        };

    Ok(hoist.unwrap_or(false))
}

#[poise::command(prefix_command, slash_command)]
pub async fn hoist(ctx: Context<'_>, hoist: Option<bool>) -> Result<(), Error> {
    let guild = ctx.guild_id().ok_or(BigBirbError::GuildOnly)?;

    match hoist {
        Some(hoist) => {
            let member = ctx.author_member().await.unwrap();

            if !member.permissions(ctx).iter().any(|p| p.manage_guild()) {
                ctx.reply("You do not have permission to change this setting.")
                    .await?;
                return Ok(());
            }

            let mut tx = ctx.data().database.begin().await?;

            sqlx::query("INSERT INTO settings (guildid, hoist_selfroles) VALUES ($1, $2) ON CONFLICT (guildid) DO UPDATE SET hoist_selfroles = EXCLUDED.hoist_selfroles")
                .bind(guild.get() as i64)
                .bind(hoist)
                .execute(&mut *tx).await?;

            tx.commit().await?;

            if hoist {
                ctx.reply("New self roles will now be automatically hoisted.")
                    .await?;
            } else {
                ctx.reply("New self roles will not be hoisted.").await?;
            }
        }
        None => {
            let s = if get_hoist_selfroles(ctx, guild).await? {
                "enabled"
            } else {
                "disabled"
            };

            ctx.reply(format!("Hoisting selfroles is {s}.")).await?;
        }
    }

    Ok(())
}

pub async fn get_banrole(ctx: Context<'_>, guild: GuildId) -> Result<Option<RoleId>, Error> {
    let db = &ctx.data().database;

    let role: Option<i64> = match sqlx::query("SELECT banrole FROM settings WHERE guildid = $1")
        .bind(guild.get() as i64)
        .fetch_one(db)
        .await
    {
        Ok(r) => r.get(0),
        Err(sqlx::Error::RowNotFound) => None,
        Err(e) => return Err(Box::new(e)),
    };

    Ok(role.map(|sf| RoleId::new(sf as u64)))
}

#[poise::command(prefix_command, slash_command)]
pub async fn banrole(ctx: Context<'_>, role: Option<Role>) -> Result<(), Error> {
    let guild = ctx.guild_id().ok_or(BigBirbError::GuildOnly)?;
    let member = ctx.author_member().await.unwrap();

    if !member.permissions(ctx).iter().any(|p| p.manage_guild()) {
        ctx.reply("You do not have permission to see or change this setting.")
            .await?;
        return Ok(());
    }

    match role {
        Some(role) => {
            let mut tx = ctx.data().database.begin().await?;

            sqlx::query("INSERT INTO settings (guildid, banrole) VALUES ($1, $2) ON CONFLICT (guildid) DO UPDATE SET banrole = EXCLUDED.banrole")
                .bind(guild.get() as i64)
                .bind(role.id.get() as i64)
                .execute(&mut *tx).await?;

            tx.commit().await?;

            common::no_ping_reply(
                &ctx,
                format!("The bot will now give banned users the role {role}."),
            )
            .await?;
        }
        None => {
            let s = match get_banrole(ctx, guild).await? {
                Some(r) => format!("{}", guild.role(ctx, r).await?),
                None => "not set".into(),
            };

            common::no_ping_reply(&ctx, format!("This server's ban role is {s}.")).await?;
        }
    }

    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    subcommands("prefix", "position", "hoist", "banrole"),
    subcommand_required
)]
pub async fn setting(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}
