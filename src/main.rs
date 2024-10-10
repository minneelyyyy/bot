use std::env;

use poise::serenity_prelude as serenity;

struct Data {}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(slash_command)]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    use std::time::Instant;

    let start = Instant::now();
    let msg = ctx.say("Pong! \u{1F3D3}").await?;

    msg.edit(ctx, poise::reply::CreateReply::default()
        .content(format!("Pong! \u{1F3D3}\nREST: {:.2?}\nGateway: {:.2?}",
            start.elapsed(), ctx.ping().await))).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::dotenv().ok();

    println!("Starting bot...");

    let token = env::var("DISCORD_BOT_TOKEN")?;
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![ping()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents).framework(framework).await;

    client.unwrap().start().await?;

    Ok(())
}
