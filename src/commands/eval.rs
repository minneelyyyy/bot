use crate::common::{self, Context, Error};
use std::io::Cursor;

/// Evaluate a program written in the Lamm programming language
#[poise::command(prefix_command, aliases("lamm"))]
pub async fn eval(ctx: Context<'_>, expr: poise::CodeBlock) -> Result<(), Error> {
	let runtime = lamm::Runtime::new(Cursor::new(expr.code), "<eval>");

	let values = runtime.values()
		.map(|res| res.map(|value| value.to_string()))
		.collect::<Result<Vec<_>, _>>();

	match values {
		Ok(values) => common::no_ping_reply(&ctx, format!("{}", values.join("\n"))).await?,
		Err(e) => common::no_ping_reply(&ctx, format!("```ansi\n\x1b[31;1merror\x1b[0m: {e}\n```")).await?,
	};

    Ok(())
}