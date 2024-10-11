use std::env;
use std::iter;
use tokio::sync::Mutex;
use poise::{serenity_prelude::{self as serenity, Colour}, CreateReply};

struct Data {
    tokens: Mutex<usize>,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Display the bot's latency to Discord's REST and Gateway APIs
#[poise::command(slash_command)]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    use std::time::Instant;

    let start = Instant::now();
    let msg = ctx.say("Pong! \u{1F3D3}").await?;
    let time = start.elapsed();

    msg.edit(ctx, poise::reply::CreateReply::default()
        .content(format!("Pong! \u{1F3D3}\nREST: {:.2?}\nGateway: {:.2?}",
            time, ctx.ping().await))).await?;

    Ok(())
}

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
        output.push_str(&format!("**Permissions**: {}\n", permissions.get_permission_names().join(", ")))
    }   

    output
}

/// Display information about a given user
#[poise::command(slash_command)]
async fn dox(
        mut ctx: Context<'_>,
        #[description = "The user to display information of"]
        user: serenity::User,
        #[rename = "permissions"]
        #[description = "Rather or not to show the user's permissions"]
        show_permissions: Option<bool>) -> Result<(), Error> {
    let user = ctx.http().get_user(user.id).await?;
    let member = if let Some(guild) = ctx.guild_id() {
        guild.member(ctx.http(), user.id).await.ok()
    } else {
        None
    };

    let embed = serenity::CreateEmbed::default()
        .title(format!("Information about {}", user.name))
        .description(get_dox_output(&mut ctx, &user, member.as_ref(), show_permissions.unwrap_or(false)))
        .colour(member.map(|m| m.colour(ctx.cache()))
            .unwrap_or(None)
            .unwrap_or(user.accent_colour.unwrap_or(Colour::from_rgb(255, 255, 255))))
        .image(user.face());

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

/// Pardner
#[poise::command(slash_command)]
async fn yeehaw(ctx: Context<'_>,
        #[min = 1]
        #[max = 64]
        width: usize,
        #[min = 1]
        #[max = 100]
        height: usize) -> Result<(), Error> {
    ctx.reply(iter::repeat("\u{1F920}".to_string().repeat(width))
        .take(height)
        .collect::<Vec<_>>()
        .join("\n")).await?;
    Ok(())
}

#[poise::command(slash_command)]
async fn wager(ctx: Context<'_>, amount: usize) -> Result<(), Error> {
    let mut wealth = ctx.data().tokens.lock().await;

    if *wealth < amount {
        ctx.reply("You do not have enough tokens to wager this amount.").await?;
        return Ok(());
    }

    if rand::random() {
        *wealth += amount;
        ctx.reply(format!("You just gained {} tokens! You now have **{}**.", amount, *wealth)).await?;
    } else {
        *wealth -= amount;
        ctx.reply(format!("You've lost **{}** tokens, you now have **{}**.", amount, *wealth)).await?;
    }

    Ok(())
}

#[poise::command(slash_command)]
async fn balance(ctx: Context<'_>) -> Result<(), Error> {
    let wealth = ctx.data().tokens.lock().await;
    ctx.reply(format!("You have **{}** tokens.", *wealth)).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::dotenv().ok();

    let token = env::var("DISCORD_BOT_TOKEN")?;
    let intents = serenity::GatewayIntents::all();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![ping(), dox(), yeehaw(), wager(), balance()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { tokens: Mutex::new(100) })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents).framework(framework).await;

    client.unwrap().start().await?;

    Ok(())
}
