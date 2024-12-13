use crate::common::{self, Context, Error};
use std::io::Cursor;

/// Evaluates a Lamm program
#[poise::command(prefix_command, aliases("lamm"))]
pub async fn eval(ctx: Context<'_>, expr: poise::CodeBlock) -> Result<(), Error> {
	let runtime = lamm::Runtime::new(Cursor::new(expr.code), "<eval>");

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
		Ok(values) => common::no_ping_reply(&ctx, format!("{values}")).await?,
		Err(e) => common::no_ping_reply(&ctx, format!("```ansi\n\x1b[31;1merror\x1b[0m: {e}\n```")).await?,
	};

    Ok(())
}