
use crate::common::{Context, Error};

use hex_color::HexColor;
use poise::serenity_prelude::{Color, EditRole};

/// Change the color of your personal role
#[poise::command(slash_command, prefix_command)]
pub async fn color(ctx: Context<'_>, color: String) -> Result<(), Error> {
    let data = ctx.data();
    let mut db = data.database.lock().await;
    let db = db.as_mut();

    let color = match HexColor::parse_rgb(&color) {
        Ok(color) => color,
        Err(e) => {
            ctx.reply(format!("Couldn't parse color: {e}")).await?;
            return Ok(());
        }
    };

    if let Some(guild) = ctx.guild_id() {
        let role = match super::get_user_role(ctx, ctx.author().id, guild, db).await? {
            Some(role) => role,
            None => {
                let role = guild.create_role(ctx,
                    EditRole::new()
                        .name(format!("{}", color.display_rgb()))
                        .colour(Color::from_rgb(color.r, color.g, color.b))).await?;

                sqlx::query("INSERT INTO selfroles (userid, roleid, guildid) VALUES ($1, $2, $3)")
                    .bind(ctx.author().id.get() as i64)
                    .bind(role.id.get() as i64)
                    .bind(guild.get() as i64)
                    .execute(db).await?;

                let member = guild.member(ctx, ctx.author().id).await?;
                member.add_role(ctx, role.clone()).await?;

                ctx.reply(format!("You have been given the {} role!", role)).await?;
                return Ok(());
            }
        };

        guild.edit_role(ctx, role, EditRole::new().colour(Color::from_rgb(color.r, color.g, color.b))).await?;

        ctx.reply("Your custom role's color has been updated!").await?;

        Ok(())
    } else {
        ctx.reply("This command must be run within a server.").await?;
        Ok(())
    }
}
