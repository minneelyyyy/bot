use crate::common::{Context, Error};

use std::collections::HashMap;
use std::borrow::Cow;

mod tokenizer;
mod parse;

fn evaluate(expr: &str) -> Result<f64, Error> {
    let tokens = tokenizer::Token::tokenize(expr)?;
    let mut tokens = tokens.iter();

    let globals = HashMap::new();
    let locals = HashMap::new();
    let mut locals = Cow::Borrowed(&locals);

    let tree = parse::ParseTree::parse(&mut tokens, &globals, &mut locals)?;

    let mut globals = HashMap::new();
    let locals = HashMap::new();
    let mut locals = Cow::Borrowed(&locals);

    tree.evaluate(&mut globals, &mut locals)
}

/// Evaluates an expression (uses Polish Notation)
#[poise::command(slash_command, prefix_command)]
pub async fn eval(ctx: Context<'_>,
                  #[rest]
                  expr: String) -> Result<(), Error>
{
    ctx.reply(format!("{}", evaluate(&expr)?)).await?;
    Ok(())
}
