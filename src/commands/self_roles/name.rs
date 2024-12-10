
use crate::common::{Context, Error};

use poise::serenity_prelude::EditRole;

/// Change the name of your personal role
#[poise::command(slash_command, prefix_command)]
pub async fn name(ctx: Context<'_>, name: String) -> Result<(), Error> {
    let mut tx = ctx.data().database.begin().await?;

    if let Some(guild) = ctx.guild_id() {
        let role = match super::get_user_role(ctx.author().id, guild, &mut *tx).await? {
            Some(role) => role,
            None => {
                let role = guild.create_role(ctx, EditRole::new().name(name)).await?;

                sqlx::query("INSERT INTO selfroles (userid, roleid, guildid) VALUES ($1, $2, $3)")
                    .bind(ctx.author().id.get() as i64)
                    .bind(role.id.get() as i64)
                    .bind(guild.get() as i64)
                    .execute(&mut *tx).await?;

                let member = guild.member(ctx, ctx.author().id).await?;
                member.add_role(ctx, role.clone()).await?;

                tx.commit().await?;

                ctx.reply(format!("You've been given the {} role!", role)).await?;

                return Ok(());
            }
        };

        guild.edit_role(ctx, role, EditRole::new().name(name)).await?;
        let role = guild.role(ctx, role).await?;

        ctx.reply(format!("{} has been updated.", role)).await?;

        Ok(())
    } else {
        ctx.reply("This command must be run within a server.").await?;
        Ok(())
    }
}
