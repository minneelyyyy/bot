
use crate::common::{self, Context, Error};

use poise::serenity_prelude as serenity;

/// Register an existing role as a user's custom role. This deletes their current self role.
#[poise::command(slash_command, prefix_command, required_permissions = "MANAGE_ROLES")]
pub async fn register(ctx: Context<'_>, user: serenity::User, role: serenity::Role) -> Result<(), Error> {
    let Some(guild) = ctx.guild_id() else {
        ctx.reply("This command must be ran within a guild.").await?;
        return Ok(());
    };

    let mut tx = ctx.data().database.begin().await?;

    if let Some(role) = super::get_user_role(user.id, guild, &mut *tx).await? {
        guild.delete_role(ctx, role).await?;
    }

    let member = guild.member(ctx, user).await?;
    member.add_role(ctx, role.id).await?;

    super::update_user_role(member.user.id, guild, role.id, &mut *tx).await?;
    tx.commit().await?;

    common::no_ping_reply(&ctx, format!("{} has been set as {}'s self role.", role, member.user)).await?;

    Ok(())
}
