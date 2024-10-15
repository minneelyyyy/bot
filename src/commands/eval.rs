use crate::common::{Context, Error};
use std::io::Cursor;

/// Evaluates a Lamm program
#[poise::command(slash_command, prefix_command)]
pub async fn eval(ctx: Context<'_>,
                  #[rest]
                  expr: String) -> Result<(), Error>
{
	let expr = expr.strip_prefix("```").and_then(|s| s.strip_suffix("```")).unwrap_or(&expr);

	let values = lamm::evaluate(Cursor::new(expr));

	let output = values.fold(Ok(String::new()), |acc, v| {
		if acc.is_err() {
			return acc;
		};

		let x = acc.unwrap();

		match v {
			Ok(v) => Ok(format!("{x}\n{v}")),
			Err(e) => Err(e),
		}
	});

	ctx.reply(output?).await?;

    Ok(())
}
