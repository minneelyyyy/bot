
use crate::common::{Context, Error};

/// Remove and delete your personal role
#[poise::command(slash_command, prefix_command)]
pub async fn disown(ctx: Context<'_>) -> Result<(), Error> {
    let guild = if let Some(guild) = ctx.guild_id() {
        guild
    } else {
        ctx.reply("This command can only be run inside of a guild.").await?;
        return Ok(());
    };

    let user = ctx.author();

    let mut tx = ctx.data().database.begin().await?;

    if let Some(role) = super::get_user_role(user.id, guild, &mut *tx).await? {
        guild.delete_role(ctx, role).await?;
        super::remove_user_role(user.id, guild, &mut *tx).await?;
        tx.commit().await?;
        ctx.reply("Your self role has been deleted.").await?;
    } else {
        ctx.reply("You currently have no self role to delete.").await?;
    }

    Ok(())
}
