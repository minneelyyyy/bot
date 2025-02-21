
use crate::common::{self, Context, Error};

use poise::serenity_prelude::EditRole;

/// Change the name of your personal role
#[poise::command(slash_command, prefix_command)]
pub async fn name(ctx: Context<'_>, #[rest] name: String) -> Result<(), Error> {
    let guild = if let Some(guild) = ctx.guild_id() {
        guild
    } else {
        ctx.reply("This command can only be run inside of a guild.").await?;
        return Ok(());
    };

    let user = ctx.author();

    let role = super::edit_role(ctx, user.id, guild, EditRole::new().name(name), &ctx.data().database).await?;
    common::no_ping_reply(&ctx, format!("{} has been updated.", guild.role(ctx, role).await?)).await?;

    Ok(())
}
