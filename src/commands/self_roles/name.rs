
use crate::common::{self, Context, Error};

use poise::serenity_prelude::{EditRole, Permissions};

/// Change the name of your personal role
#[poise::command(slash_command, prefix_command)]
pub async fn name(ctx: Context<'_>, name: String) -> Result<(), Error> {
    let guild = if let Some(guild) = ctx.guild_id() {
        guild
    } else {
        ctx.reply("This command can only be run inside of a guild.").await?;
        return Ok(());
    };

    let user = ctx.author();

    let mut tx = ctx.data().database.begin().await?;

    if let Some(role) = super::get_user_role(user.id, guild, &mut *tx).await? {
        let role = guild.role(ctx, role).await?;
        guild.edit_role(ctx, role.id, EditRole::new().name(name)).await?;
        common::no_ping_reply(&ctx, format!("{} has been updated.", role)).await?;
    } else {
        let role = guild.create_role(ctx, EditRole::new().name(name).permissions(Permissions::empty())).await?;
        super::update_user_role(user.id, guild, role.id, &mut *tx).await?;
        let member = guild.member(ctx, user).await?;
        member.add_role(ctx, role.id).await?;
        tx.commit().await?;
        common::no_ping_reply(&ctx, format!("{} has been given the new role {}.", user, role)).await?;
    }

    Ok(())
}
