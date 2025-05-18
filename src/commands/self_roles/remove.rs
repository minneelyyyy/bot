
use crate::common::{self, Context, Error};

use poise::serenity_prelude as serenity;

#[poise::command(slash_command, prefix_command, required_permissions = "MANAGE_ROLES")]
pub async fn remove(ctx: Context<'_>, user: serenity::User) -> Result<(), Error> {
    let Some(guild) = ctx.guild_id() else {
        ctx.reply("This command must be ran within a guild.").await?;
        return Ok(());
    };

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