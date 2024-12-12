use poise::Command;

mod ping;
mod dox;
mod yeehaw;
mod gambling;
mod eval;
mod self_roles;
mod settings;

use crate::common::{Data, Error};

pub fn commands() -> Vec<Command<Data, Error>> {
    vec![
        ping::ping(),
        dox::dox(),
        yeehaw::yeehaw(),
        gambling::balance::balance(),
        gambling::give::give(),
        gambling::wager::wager(),
        gambling::daily::daily(),
        gambling::leaderboard::leaderboard(),
        gambling::shop::buy(),
        eval::eval(),
        self_roles::role(),
        settings::setting(),
    ]
}
