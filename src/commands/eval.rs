use crate::common::{self, Context, Error};
use std::io::Cursor;

/// Evaluates a Lamm program
#[poise::command(slash_command, prefix_command, aliases("lamm"))]
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

	let values = runtime.values()
		.map(|res| res.map(|value| value.to_string()))
		.collect::<Result<Vec<_>, _>>();

	match values {
		Ok(values) => common::no_ping_reply(&ctx, format!("{}", values.join("\n"))).await?,
		Err(e) => common::no_ping_reply(&ctx, format!("```ansi\n\x1b[31;1merror\x1b[0m: {e}\n```")).await?,
	};

    Ok(())
}