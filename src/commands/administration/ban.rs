use crate::commands::settings;
use crate::common::{BigBirbError, Context, Error};

use poise::serenity_prelude as serenity;

/// Ban a user
#[poise::command(slash_command, prefix_command)]
pub async fn ban(
    ctx: Context<'_>,
    user: serenity::User,
    #[rest] reason: Option<String>,
) -> Result<(), Error> {
    let guild = ctx.guild_id().ok_or(BigBirbError::GuildOnly)?;

    if let Some(role) = settings::get_banrole(ctx, guild).await? {
        let member = guild.member(&ctx, user.id).await?;
        member.add_role(ctx, &role).await?;
    };

    if let Some(reason) = reason {
        ctx.reply(format!("{user} has been banned for {reason}."))
            .await?;
    } else {
        ctx.reply(format!("{user} has been banned.")).await?;
    }
    Ok(())
}

/// Unban a user
#[poise::command(slash_command, prefix_command)]
pub async fn unban(ctx: Context<'_>, user: serenity::User) -> Result<(), Error> {
    let guild = ctx.guild_id().ok_or(BigBirbError::GuildOnly)?;

    if let Some(role) = settings::get_banrole(ctx, guild).await? {
        let member = guild.member(&ctx, user.id).await?;
        member.remove_role(ctx, &role).await?;
    };

    ctx.reply(format!("{user} has been unbanned.")).await?;
    Ok(())
}
