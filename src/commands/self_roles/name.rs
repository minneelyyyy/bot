
use crate::common::{self, Context, Error, BigBirbError};
use poise::serenity_prelude::{EditRole, User, GuildId, RoleId};

pub async fn change_user_role_name(ctx: Context<'_>, user: &User, guild: GuildId, name: String) -> Result<RoleId, Error> {
    let mut tx = ctx.data().database.begin().await?;
    let role = super::edit_role(ctx, user.id, guild, EditRole::new().name(name), &mut *tx).await?;
    tx.commit().await?;

    Ok(role)
}

/// Change the name of your personal role
#[poise::command(slash_command, prefix_command, )]
pub async fn name(ctx: Context<'_>, #[rest] name: String) -> Result<(), Error> {
    let guild = ctx.guild_id().ok_or(BigBirbError::GuildOnly)?;
    let user = ctx.author();

    let role = guild.role(ctx, change_user_role_name(ctx, user, guild, name).await?).await?;
    common::no_ping_reply(&ctx, format!("{role} has been updated.")).await?;

    Ok(())
}
