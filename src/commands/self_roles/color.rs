
use crate::common::{Context, Error};

use hex_color::HexColor;
use poise::serenity_prelude::{colours, Color, EditRole};

/// Change the color of your personal role
#[poise::command(slash_command, prefix_command)]
pub async fn color(ctx: Context<'_>, color: String) -> Result<(), Error> {
    let color = match color.to_lowercase().as_str() {
        "reset" | "default" => colours::roles::DEFAULT,
        "teal" => colours::roles::TEAL,
        "dark teal" => colours::roles::DARK_TEAL,
        "green" => colours::roles::GREEN,
        "dark green" => colours::roles::DARK_GREEN,
        "blue" => colours::roles::BLUE,
        "dark blue" => colours::roles::DARK_BLUE,
        "purple" => colours::roles::PURPLE,
        "dark purple" => colours::roles::DARK_PURPLE,
        "magenta" => colours::roles::MAGENTA,
        "dark magenta" => colours::roles::DARK_MAGENTA,
        "gold" => colours::roles::GOLD,
        "dark gold" => colours::roles::DARK_GOLD,
        "orange" => colours::roles::DARK_ORANGE,
        "dark orange" => colours::roles::DARK_ORANGE,
        "red" => colours::roles::RED,
        "dark red" => colours::roles::DARK_RED,
        "lighter grey" | "lighter gray" => colours::roles::LIGHTER_GREY,
        "light grey" | "light gray" => colours::roles::LIGHT_GREY,
        "dark grey" | "dark gray" => colours::roles::DARK_GREY,
        "darker grey" | "darker gray" => colours::roles::DARKER_GREY,
        "white" => Color::new(0xffffff),
        "black" => Color::new(0x010101),
        "blurple" => Color::BLURPLE,
        "yellow" => Color::new(0xe4f10a),
        hex => match HexColor::parse_rgb(hex) {
            Ok(color) => Color::from_rgb(color.r, color.g, color.b),
            Err(_) => {
                ctx.reply(format!("Unable to parse `{}` as a color.", color)).await?;
                return Ok(());
            }
        }
    };

    let mut tx = ctx.data().database.begin().await?;

    if let Some(guild) = ctx.guild_id() {
        match super::get_user_role(ctx.author().id, guild, &mut *tx).await? {
            Some(role) => {
                guild.edit_role(ctx, role, EditRole::new().colour(color)).await?;
                let role = guild.role(ctx, role).await?;

                ctx.reply(format!("{}'s color has been updated!", role)).await?;

                Ok(())
            },
            None => {
                let role = guild.create_role(ctx,
                    EditRole::new()
                        .name(format!("#{:06x}", color.0))
                        .colour(color)).await?;

                sqlx::query("INSERT INTO selfroles (userid, roleid, guildid) VALUES ($1, $2, $3)")
                    .bind(ctx.author().id.get() as i64)
                    .bind(role.id.get() as i64)
                    .bind(guild.get() as i64)
                    .execute(&mut *tx).await?;

                let member = guild.member(ctx, ctx.author().id).await?;
                member.add_role(ctx, role.clone()).await?;

                ctx.reply(format!("You have been given the {} role!", role)).await?;
                return Ok(());
            }
        }
    } else {
        ctx.reply("This command must be run within a server.").await?;
        Ok(())
    }
}
