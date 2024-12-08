
use crate::common::{Context, Error};

use poise::serenity_prelude::EditRole;

/// Change the name of your personal role
#[poise::command(slash_command, prefix_command)]
pub async fn name(ctx: Context<'_>, name: String) -> Result<(), Error> {
    let data = ctx.data();
    let mut db = data.database.lock().await;
    let db = db.as_mut();

    if let Some(guild) = ctx.guild_id() {
        let role = match super::get_user_role(ctx, ctx.author().id, guild, db).await? {
            Some(role) => role,
            None => {
                let role = guild.create_role(ctx, EditRole::new().name(name)).await?;

                sqlx::query("INSERT INTO selfroles (userid, roleid, guildid) VALUES ($1, $2, $3)")
                    .bind(ctx.author().id.get() as i64)
                    .bind(role.id.get() as i64)
                    .bind(guild.get() as i64)
                    .execute(db).await?;

                let member = guild.member(ctx, ctx.author().id).await?;
                member.add_role(ctx, role.clone()).await?;

                ctx.reply(format!("You've been given the {} role!", role)).await?;

                return Ok(());
            }
        };

        guild.edit_role(ctx, role, EditRole::new().name(name)).await?;

        ctx.reply("Your custom role's name has been updated!").await?;

        Ok(())
    } else {
        ctx.reply("This command must be run within a server.").await?;
        Ok(())
    }
}
