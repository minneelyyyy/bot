use crate::common::{Context, Error};

mod tokenizer;
mod parse;

fn evaluate(expr: &str) -> Result<f64, Error> {
    let tokens = tokenizer::Token::tokenize(expr)?;
    let mut tokens = tokens.iter();

    let tree = parse::ParseTree::new(&mut tokens)?;

    Ok(tree.evaluate())
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
