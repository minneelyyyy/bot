
use crate::common::{self, Context, Error};

use poise::serenity_prelude as serenity;

/// force remove someone's role (this does not delete the role)
#[poise::command(slash_command, prefix_command, required_permissions = "MANAGE_ROLES")]
pub async fn remove(ctx: Context<'_>, user: serenity::User) -> Result<(), Error> {
    let data = ctx.data();
    let mut db = data.database.lock().await;
    let db = db.as_mut();

    if let Some(guild) = ctx.guild_id() {
        match super::get_user_role(user.id, guild, db).await? {
            Some(role) => {
                sqlx::query("DELETE FROM selfroles WHERE userid = $1 AND guildid = $2")
                    .bind(user.id.get() as i64)
                    .bind(guild.get() as i64)
                    .execute(db).await?;

                let member = guild.member(ctx, user.id).await?;
                
                member.remove_role(ctx, role).await?;
                let role = guild.role(ctx, role).await?;

                common::no_ping_reply(&ctx, format!("{} has had {} removed.", user, role)).await?;

                Ok(())
            },
            None => {
                common::no_ping_reply(&ctx, format!("{} does not have a personal role to remove.", user)).await?;
                Ok(())
            }
        }
    } else {
        ctx.reply("This command can only be run in a guild!").await?;
        Ok(())
    }
}
