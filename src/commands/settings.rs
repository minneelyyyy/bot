use crate::common::{Context, Error};

use poise::serenity_prelude::{Role, RoleId, GuildId};
use sqlx::Row;

async fn get_prefix(ctx: Context<'_>, guild: GuildId) -> Result<Option<String>, Error> {
    let db = &ctx.data().database;

    let prefix: Option<String> = sqlx::query("SELECT prefix FROM settings WHERE guildid = $1")
        .bind(guild.get() as i64)
        .fetch_one(db).await?.get(0);

    Ok(prefix.or(ctx.data().prefix.clone()))
}

#[poise::command(prefix_command, slash_command)]
async fn prefix(ctx: Context<'_>, prefix: Option<String>) -> Result<(), Error> {
    let guild = match ctx.guild_id() {
        Some(g) => g,
        None => {
            ctx.reply("This command must be ran within a guild.").await?;
            return Ok(());
        }
    };

    match prefix {
        Some(prefix) => {
            if !ctx.author_member().await.unwrap().permissions.iter().any(|p| p.manage_guild()) {
                ctx.reply("You do not have permission to change this setting.").await?;
                return Ok(());
            }

            let mut tx = ctx.data().database.begin().await?;

            sqlx::query("INSERT INTO settings (guildid, prefix) VALUES ($1, $2) ON CONFLICT (guildid) DO UPDATE SET prefix = EXCLUDED.prefix")
                .bind(guild.get() as i64)
                .bind(&prefix)
                .execute(&mut *tx).await?;
        
            tx.commit().await?;
        
            ctx.reply(format!("This server's custom prefix has been updated to `{prefix}`.")).await?;
        }
        None => {
            let s = get_prefix(ctx, guild).await?.map(|s| format!("`{s}`")).unwrap_or("not set".into());
            ctx.reply(format!("This server's command prefix is {s}.")).await?;
        }
    }

    Ok(())
}

pub async fn get_positional_role(ctx: Context<'_>, guild: GuildId) -> Result<Option<RoleId>, Error> {
    let db = &ctx.data().database;

    let role: Option<i64> = sqlx::query("SELECT positional_role FROM settings WHERE guildid = $1")
        .bind(guild.get() as i64)
        .fetch_one(db).await?.get(0);

    Ok(role.map(|sf| RoleId::new(sf as u64)))
}

#[poise::command(prefix_command, slash_command)]
pub async fn position(ctx: Context<'_>, role: Option<Role>) -> Result<(), Error> {
    let guild = match ctx.guild_id() {
        Some(g) => g,
        None => {
            ctx.reply("This command must be ran within a guild.").await?;
            return Ok(());
        }
    };

    if !ctx.author_member().await.unwrap().permissions.iter().any(|p| p.manage_guild()) {
        ctx.reply("You do not have permission to see or change this setting.").await?;
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

            ctx.reply(format!("The bot will now place newly created self roles below `{role}`.")).await?;
        }
        None => {
            let s = match get_positional_role(ctx, guild).await? {
                Some(r) => format!("{}", guild.role(ctx, r).await?),
                None => "not set".into()
            };

            ctx.reply(format!("This server's positional role is {s}.")).await?;
        }
    }

    Ok(())
}

#[poise::command(prefix_command, slash_command, subcommands("prefix", "position"), subcommand_required)]
pub async fn setting(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}