use crate::common::{self, BigBirbError, Context, Error};

use once_cell::sync::Lazy;
use std::collections::HashMap;

use hex_color::HexColor;
use poise::serenity_prelude::{colours, Color, EditRole, GuildId, RoleId, User};

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
        ("cyan", Color::new(0xc58ffff)),
        ("turqouise", Color::new(0x1bc6c6)),
        ("pink", Color::new(0xffacac)),
        ("hot pink", Color::new(0xa52a67)),
        ("light orange", Color::new(0xffc88a)),
        ("light yellow", Color::new(0xfbff8a)),
        ("light green", Color::new(0xc1ff8a)),
        ("light blue", Color::new(0x8afbff)),
        ("light purple", Color::new(0xffc0f5)),
        ("light cyan", Color::new(0xc0ffff)),
    ])
});

async fn autocomplete_colors<'a>(
    _ctx: Context<'_>,
    partial: &'a str,
) -> impl Iterator<Item = &'static str> + use<'a> {
    COLORS
        .clone()
        .into_keys()
        .filter(move |x| x.split_whitespace().any(|x| x.starts_with(partial)))
}

pub fn parse_color(s: &str) -> Result<Color, Error> {
    let color = if let Some(named) = COLORS.get(s) {
        named.clone()
    } else {
        let rgb = HexColor::parse_rgb(&s)?;
        Color::from_rgb(rgb.r, rgb.g, rgb.b)
    };

    Ok(color)
}

pub async fn change_user_role_color(
    ctx: Context<'_>,
    user: &User,
    guild: GuildId,
    color: Color,
) -> Result<RoleId, Error> {
    let mut tx = ctx.data().database.begin().await?;
    let role =
        super::edit_role(ctx, user.id, guild, EditRole::new().colour(color), &mut *tx).await?;
    tx.commit().await?;

    Ok(role)
}

/// Change the color of your personal role
#[poise::command(slash_command, prefix_command)]
pub async fn color(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_colors"]
    #[rest]
    color: String,
) -> Result<(), Error> {
    let guild = ctx.guild_id().ok_or(BigBirbError::GuildOnly)?;
    let user = ctx.author();
    let color = parse_color(&color)?;

    let role = guild
        .role(ctx, change_user_role_color(ctx, &user, guild, color).await?)
        .await?;
    common::no_ping_reply(&ctx, format!("{role}'s color has been updated.")).await?;

    Ok(())
}
