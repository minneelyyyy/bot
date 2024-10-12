use crate::common::{Context, Error};

use poise::serenity_prelude as serenity;
use serenity::Colour;

fn get_dox_output(ctx: &mut Context<'_>,
                  user: &serenity::User,
                  member: Option<&serenity::Member>,
                  show_permissions: bool) -> String {
    let mut output = String::new();

    if user.bot {
        output.push_str("This user is a bot.\n");
    }

    output.push_str(&format!("**User ID**: {}\n", user.id));

    if let Some(locale) = &user.locale {
        output.push_str(&format!("**Locale**: {locale}\n"));
    }

    if let Some(verified) = &user.verified {
        output.push_str(&format!("**Verified**: {verified}\n"));
    }

    output.push_str(&format!("**Account Created**: {}\n", user.created_at()));

    if let Some(Some(join_date)) = member.as_ref().map(|m| m.joined_at) {
        output.push_str(&format!("**Joined this Server at**: {join_date}\n"));
    }

    if let Some(Some(premium_since)) = member.as_ref().map(|m| m.premium_since) {
        output.push_str(
            &format!("**Boosting this Server**: Yes\n**Boosting since**: {premium_since}\n"));
    }

    if let Some(Ok(permissions)) = member.map(|m| m.permissions(ctx)).filter(|_| show_permissions) {
        output.push_str(&format!("**Permissions**: {}\n",
                                 permissions.get_permission_names().join(", ")))
    }

    output
}

/// Display information about a given user
#[poise::command(slash_command, prefix_command)]
pub async fn dox(mut ctx: Context<'_>,
                 #[description = "The user to display information of"]
                 user: serenity::User,
                 #[rename = "permissions"]
                 #[description = "Rather or not to show the user's permissions"]
                 show_permissions: Option<bool>) -> Result<(), Error>
{
    let user = ctx.http().get_user(user.id).await?;
    let member = if let Some(guild) = ctx.guild_id() {
        guild.member(ctx.http(), user.id).await.ok()
    } else {
        None
    };

    let embed = serenity::CreateEmbed::default()
        .title(format!("Information about {}", user.name))
        .description(get_dox_output(
            &mut ctx, &user, member.as_ref(), show_permissions.unwrap_or(false)))
        .colour(member.map(|m| m.colour(ctx.cache()))
            .unwrap_or(None)
            .unwrap_or(user.accent_colour.unwrap_or(Colour::from_rgb(255, 255, 255))))
        .image(user.face());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}
