
use crate::common::{Context, Error};

use once_cell::sync::Lazy;
use std::collections::HashMap;

use hex_color::HexColor;
use poise::serenity_prelude::{colours, Color, EditRole};

static COLORS: Lazy<HashMap<&'static str, Color>> = Lazy::new(|| {
    HashMap::from([
        ("reset", colours::roles::DEFAULT),
        ("default", colours::roles::DEFAULT),
        ("teal", colours::roles::TEAL),
        ("dark teal", colours::roles::DARK_TEAL),
        ("green", colours::roles::GREEN),
        ("dark green", colours::roles::DARK_GREEN),
        ("blue", colours::roles::BLUE),
        ("dark blue", colours::roles::DARK_BLUE),
        ("purple", colours::roles::PURPLE),
        ("dark purple", colours::roles::DARK_PURPLE),
        ("magenta", colours::roles::MAGENTA),
        ("dark magenta", colours::roles::DARK_MAGENTA),
        ("gold", colours::roles::GOLD),
        ("dark gold", colours::roles::DARK_GOLD),
        ("orange", colours::roles::DARK_ORANGE),
        ("dark orange", colours::roles::DARK_ORANGE),
        ("red", colours::roles::RED),
        ("dark red", colours::roles::DARK_RED),
        ("lighter grey", colours::roles::LIGHTER_GREY),
        ("lighter gray", colours::roles::LIGHTER_GREY),
        ("light grey", colours::roles::LIGHT_GREY),
        ("light gray", colours::roles::LIGHT_GREY),
        ("dark grey", colours::roles::DARK_GREY),
        ("dark gray", colours::roles::DARK_GREY),
        ("darker grey", colours::roles::DARKER_GREY),
        ("darker gray", colours::roles::DARKER_GREY),
        ("white", Color::new(0xffffff)),
        ("black", Color::new(0x010101)),
        ("blurple", Color::BLURPLE),
        ("yellow", Color::new(0xe4f10a)),
    ])
});

async fn autocomplete_colors<'a>(
    _ctx: Context<'_>,
    partial: &'a str,
) -> impl Iterator<Item = &'static str> + use<'a> {
    COLORS.clone().into_keys().filter(move |x| x.split_whitespace().any(|x| x.starts_with(partial)))
}

/// Change the color of your personal role
#[poise::command(slash_command, prefix_command)]
pub async fn color(ctx: Context<'_>, #[autocomplete = "autocomplete_colors"] color: String) -> Result<(), Error> {
    let color = if let Some(named) = COLORS.get(color.as_str()) {
        named.clone()
    } else {
        let rgb = HexColor::parse_rgb(&color)?;
        Color::from_rgb(rgb.r, rgb.g, rgb.b)
    };

    if let Some(guild) = ctx.guild_id() {
        let mut tx = ctx.data().database.begin().await?;

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
                
                tx.commit().await?;

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
