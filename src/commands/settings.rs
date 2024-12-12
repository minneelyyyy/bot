use crate::common::{Context, Error};

#[poise::command(prefix_command, slash_command, required_permissions = "MANAGE_GUILD")]
async fn prefix(ctx: Context<'_>, prefix: String) -> Result<(), Error> {
    let guild = match ctx.guild_id() {
        Some(g) => g,
        None => {
            ctx.reply("This command must be ran within a guild.").await?;
            return Ok(());
        }
    };

    let mut tx = ctx.data().database.begin().await?;

    sqlx::query("INSERT INTO settings (guildid, prefix) VALUES ($1, $2) ON CONFLICT (guildid) DO UPDATE SET prefix = EXCLUDED.prefix")
        .bind(guild.get() as i64)
        .bind(&prefix)
        .execute(&mut *tx).await?;

    tx.commit().await?;

    ctx.reply(format!("This server's custom prefix has been updated to `{prefix}`.")).await?;

    Ok(())
}

#[poise::command(prefix_command, slash_command, subcommands("prefix"), subcommand_required)]
pub async fn setting(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}