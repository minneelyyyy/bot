
use crate::common::{self, Context, Error, BigBirbError};

use poise::serenity_prelude::{User, Role};

/// Change the name of a user's personal role
#[poise::command(slash_command, prefix_command, required_permissions = "MANAGE_ROLES")]
pub async fn name(ctx: Context<'_>, user: User, #[rest] name: String) -> Result<(), Error> {
    let guild = ctx.guild_id().ok_or(BigBirbError::GuildOnly)?;

    let role = guild.role(ctx, super::name::change_user_role_name(ctx, &user, guild, name).await?).await?;
    common::no_ping_reply(&ctx, format!("{role} has been updated.")).await?;

    Ok(())
}

/// Change the name of a user's personal role
#[poise::command(slash_command, prefix_command, required_permissions = "MANAGE_ROLES")]
pub async fn color(ctx: Context<'_>, user: User, color: String) -> Result<(), Error> {
    let guild = ctx.guild_id().ok_or(BigBirbError::GuildOnly)?;
    let color = super::color::parse_color(&color)?;

    let role = guild.role(ctx, super::color::change_user_role_color(ctx, &user, guild, color).await?).await?;
    common::no_ping_reply(&ctx, format!("{role}'s color has been updated.")).await?;

    Ok(())
}

/// Change a user's role name and color at once
#[poise::command(slash_command, prefix_command, required_permissions = "MANAGE_ROLES")]
pub async fn set(ctx: Context<'_>, user: User, color: String, #[rest] name: String) -> Result<(), Error> {
    let guild = ctx.guild_id().ok_or(BigBirbError::GuildOnly)?;
    let color = super::color::parse_color(&color)?;

    super::color::change_user_role_color(ctx, &user, guild, color).await?;
    let role = guild.role(ctx, super::name::change_user_role_name(ctx, &user, guild, name).await?).await?;

    common::no_ping_reply(&ctx, format!("{role} has been updated.")).await?;

    Ok(())
}

/// Remove and delete user's self role
#[poise::command(slash_command, prefix_command, aliases("disown"), required_permissions = "MANAGE_ROLES")]
pub async fn remove(ctx: Context<'_>, user: User) -> Result<(), Error> {
    let guild = ctx.guild_id().ok_or(BigBirbError::GuildOnly)?;
    let mut tx = ctx.data().database.begin().await?;

    if let Some(role) = super::get_user_role(user.id, guild, &mut *tx).await? {
        guild.delete_role(ctx, role).await?;
        super::remove_role(role, guild, &mut *tx).await?;
        tx.commit().await?;
        common::no_ping_reply(&ctx, format!("{}'s self role has been deleted.", user)).await?;
    } else {
        common::no_ping_reply(&ctx, format!("{} has no self role.", user)).await?;
    }

    Ok(())
}

/// Give a user an existing role as their self role
#[poise::command(slash_command, prefix_command, required_permissions = "MANAGE_ROLES")]
pub async fn give(ctx: Context<'_>, user: User, role: Role, force: Option<bool>) -> Result<(), Error> {
    let guild = ctx.guild_id().ok_or(BigBirbError::GuildOnly)?;
    let force = force.unwrap_or(false);
    let member = guild.member(ctx, user).await?;

    let mut tx = ctx.data().database.begin().await?;

    if force {
        // delete existing self role for user
        if let Some(original) = super::get_user_role(member.user.id, guild, &mut *tx).await? {
            guild.delete_role(ctx, original).await?;
            super::remove_role(role.id, guild, &mut *tx).await?;
        }

        // remove role from another user if it is already registered as their self role
        if let Some(user) = super::get_user_by_role(role.id, guild, &mut *tx).await? {
            let m = guild.member(ctx, user).await?;
            m.remove_role(ctx, role.id).await?;
            super::remove_role(role.id, guild, &mut *tx).await?;
        }

        super::update_user_role(member.user.id, guild, role.id, &mut *tx).await?;
        member.add_role(ctx, role.id).await?;
    } else {
        if let Some(original) = super::get_user_role(member.user.id, guild, &mut *tx).await? {
            common::no_ping_reply(&ctx, format!("{original} is already set as this user's self role, enable force to overwrite.")).await?;
            return Ok(());
        }

        if let Some(owner) = super::get_user_by_role(role.id, guild, &mut *tx).await? {
            common::no_ping_reply(&ctx, format!("{role} is already owned by {owner}, enable force to overwrite.")).await?;
            return Ok(());
        }

        super::update_user_role(member.user.id, guild, role.id, &mut *tx).await?;
    }

    tx.commit().await?;

    common::no_ping_reply(&ctx, format!("{member} has been given the self role {role}.")).await?;

    Ok(())
}