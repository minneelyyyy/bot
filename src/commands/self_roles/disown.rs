
use crate::common::{Context, Error};

/// Remove and delete your personal role
#[poise::command(slash_command, prefix_command)]
pub async fn disown(ctx: Context<'_>) -> Result<(), Error> {
    let db = &ctx.data().database;

    if let Some(guild) = ctx.guild_id() {
        if let Some(role) = super::get_user_role(ctx.author().id, guild, db).await? {
            guild.delete_role(ctx, role).await?;

            sqlx::query("DELETE FROM selfroles WHERE roleid = $1")
                .bind(role.get() as i64)
                .execute(db).await?;

            ctx.reply("Your role has been successfully removed.").await?;

            Ok(())
        } else {
            ctx.reply("You do not currently have a personal role to remove.").await?;
            Ok(())
        }
    } else {
        ctx.reply("This command must be called within a server.").await?;
        Ok(())
    }
}
