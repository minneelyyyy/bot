use poise::Command;

mod ping;
mod dox;
mod yeehaw;
mod gambling;
mod eval;
pub mod self_roles;
mod settings;
mod version;

use crate::common::{Data, Error, Context};

/// Display a help menu
#[poise::command(prefix_command, slash_command)]
async fn help(ctx: Context<'_>,
    #[description = "Specific command to get help with"]
    #[rest]
    command: Option<String>) -> Result<(), Error>
{
    poise::builtins::help(ctx, command.as_deref(), poise::builtins::HelpConfiguration::default()).await?;
    Ok(())
}

pub fn commands() -> Vec<Command<Data, Error>> {
    vec![
        help(),
        ping::ping(),
        dox::dox(),
        yeehaw::yeehaw(),
        gambling::balance::balance(),
        gambling::give::give(),
        gambling::wager::wager(),
        gambling::daily::daily(),
        gambling::leaderboard::leaderboard(),
        gambling::shop::buy(),
        gambling::blackjack::blackjack(),
        eval::eval(),
        self_roles::role(),
        settings::setting(),
        version::version(),
    ]
}
