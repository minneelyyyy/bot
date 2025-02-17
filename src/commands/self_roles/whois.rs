
use crate::common::{self, Context, Error};

use poise::serenity_prelude as serenity;
use serenity::UserId;
use sqlx::Row;

/// Let you know who is the owner of a role.
#[poise::command(slash_command, prefix_command)]
pub async fn whois(ctx: Context<'_>, role: serenity::Role) -> Result<(), Error> {
    let db = &ctx.data().database;

    if let Some(guild) = ctx.guild_id() {
        let user = match sqlx::query("SELECT userid FROM selfroles WHERE roleid = $1")
            .bind(role.id.get() as i64)
            .fetch_one(db).await
        {
            Ok(row) => UserId::new(row.try_get::<i64, usize>(0)? as u64),
            Err(sqlx::Error::RowNotFound) => {
                ctx.reply("This role is not owned by anyone.").await?;
                return Ok(());
            }
            Err(e) => return Err(Box::new(e)),
        };

        let member = guild.member(ctx, user).await?;

        common::no_ping_reply(&ctx, format!("{} owns this role.", member)).await?;
    } else {
        ctx.reply("This command must be used within a server!").await?;
    }

    Ok(())
}
