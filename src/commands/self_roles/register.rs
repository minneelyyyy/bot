
use crate::common::{self, Context, Error};

use poise::serenity_prelude as serenity;

/// Register an existing role as a user's custom role
#[poise::command(slash_command, prefix_command, required_permissions = "MANAGE_ROLES")]
pub async fn register(ctx: Context<'_>, user: serenity::User, role: serenity::Role) -> Result<(), Error> {
    let mut tx = ctx.data().database.begin().await?;

    if let Some(guild) = ctx.guild_id() {
        match super::get_user_role(user.id, guild, &mut *tx).await? {
            Some(role) => {
                let role = guild.role(ctx, role).await?;
                common::no_ping_reply(&ctx, format!("{} already has the role {} registered.", user, role)).await?;
            },
            None => {
                sqlx::query("INSERT INTO selfroles (userid, guildid, roleid) VALUES ($1, $2, $3)")
                    .bind(user.id.get() as i64)
                    .bind(guild.get() as i64)
                    .bind(role.id.get() as i64)
                    .execute(&mut *tx).await?;

                let member = guild.member(ctx, user.id).await?;
                member.add_role(ctx, role.id).await?;
        
                common::no_ping_reply(&ctx, format!("{} now has {} set as their personal role.", user, role)).await?;
            }
        }
    } else {
        ctx.reply("This command can only be run in a guild!").await?;
    }

    tx.commit().await?;
    Ok(())
}
