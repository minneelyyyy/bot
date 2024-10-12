use crate::{Data, Error};
use poise::Command;

mod ping;
mod dox;
mod yeehaw;
mod gambling;
mod eval;

pub fn commands() -> Vec<Command<Data, Error>> {
    vec![
        ping::ping(),
        dox::dox(),
        yeehaw::yeehaw(),
        gambling::balance::balance(),
        gambling::give::give(),
        gambling::wager::wager(),
        eval::eval(),
    ]
}
