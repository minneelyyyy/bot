use crate::common::{Context, Error};
use std::io::Cursor;

/// Evaluates a Lamm program
#[poise::command(slash_command, prefix_command)]
pub async fn eval(ctx: Context<'_>,
                  #[rest]
                  expr: String) -> Result<(), Error>
{
	let expr = if expr.starts_with("```\n") {
		expr.strip_prefix("```\n")
		.and_then(|s| s.strip_suffix("```"))
		.unwrap_or(&expr)
	} else if expr.starts_with("```") {
		expr.strip_prefix("```")
		.and_then(|s| s.strip_suffix("```"))
		.unwrap_or(&expr)
	} else if expr.starts_with('`') {
		expr.strip_prefix("`")
			.and_then(|s| s.strip_suffix("`"))
			.unwrap_or(&expr)
	} else {
		&expr
	};

	let runtime = lamm::Runtime::new(Cursor::new(expr), "<eval>");

	let values = runtime.values().fold(Ok(String::new()), |acc, v| {
		if acc.is_err() {
			return acc;
		};

		let x = acc.unwrap();

		match v {
			Ok(lamm::Value::Nil) => Ok(x),
			Ok(v) => Ok(format!("{x}\n{v}")),
			Err(e) => Err(e),
		}
	});

	match values {
		Ok(values) => ctx.reply(format!("{values}")).await,
		Err(e) => ctx.reply(format!("```ansi\n\x1b[31;1merror\x1b[0m: {e}\n```")).await,
	}?;

    Ok(())
}