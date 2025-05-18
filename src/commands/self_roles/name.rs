
use crate::common::{self, Context, Error, BigBirbError};

use poise::serenity_prelude::EditRole;

/// Change the name of your personal role
#[poise::command(slash_command, prefix_command)]
pub async fn name(ctx: Context<'_>, #[rest] name: String) -> Result<(), Error> {
    let guild = ctx.guild_id().ok_or(BigBirbError::GuildOnly)?;
    let user = ctx.author();

    let mut tx = ctx.data().database.begin().await?;
    let role = super::edit_role(ctx, user.id, guild, EditRole::new().name(name), &mut *tx).await?;
    tx.commit().await?;

    common::no_ping_reply(&ctx, format!("{} has been updated.", guild.role(ctx, role).await?)).await?;


    Ok(())
}
