
use crate::common::{Context, Error};

use poise::serenity_prelude as serenity;

/// Register an existing role as a user's custom role
#[poise::command(slash_command, prefix_command, required_permissions = "MANAGE_ROLES")]
pub async fn register(ctx: Context<'_>, user: Option<serenity::User>, role: serenity::Role) -> Result<(), Error> {
    let data = ctx.data();
    let mut db = data.database.lock().await;
    let db = db.as_mut();

    let user = user.as_ref().unwrap_or(ctx.author());

    if let Some(guild) = ctx.guild().map(|g| g.id) {
        sqlx::query("INSERT INTO selfroles (userid, roleid, guildid) VALUES ($1, $2, $3) ON CONFLICT (userid) DO UPDATE SET roleid = EXCLUDED.roleid")
            .bind(user.id.get() as i64)
            .bind(role.id.get() as i64)
            .bind(guild.get() as i64)
            .execute(db).await?;

        ctx.reply(format!("**{}** now has **{}** set as their personal role.", user.display_name(), role.name)).await?;

        Ok(())
    } else {
        ctx.reply("This command can only be run in a guild!").await?;
        Ok(())
    }
}
