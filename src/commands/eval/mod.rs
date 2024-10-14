use crate::common::{Context, Error};

use std::str::FromStr;
use std::fmt::Display;

mod tokenizer;
mod parser;
mod executor;

/// Evaluates an expression (uses Polish Notation)
#[poise::command(slash_command, prefix_command)]
pub async fn eval(ctx: Context<'_>,
                  #[rest]
                  expr: String) -> Result<(), Error>
{
    let expr = expr.strip_prefix("```").and_then(|s| s.strip_suffix("```")).unwrap_or(&expr);

    let tok = tokenizer::Tokenizer::from_str(&expr).unwrap(); // Error type is () and never returned
    let exprs = parser::Parser::new(tok);
    let exec = executor::Executor::new(exprs);

    let values: Vec<Value> = exec.collect::<Result<_, executor::RuntimeError>>()?;

    let reply: String = values.iter().fold(String::new(), |acc, s| acc + &format!("{s}\n"));

    ctx.reply(reply).await?;

    Ok(())
}

#[derive(Clone, Debug)]
enum Type {
    Float,
    Int,
    Bool,
    String,
    Nil,
    Any,
    Function(Box<Type>, Vec<Type>),
}

#[derive(Clone, Debug)]
pub enum Value {
    Float(f64),
    Int(i64),
    Bool(bool),
    String(String),
    Nil,
}

impl std::ops::Add for Value {
    type Output = Option<Value>;

    fn add(self, rhs: Self) -> Self::Output {
        match self {
            Self::Int(x) => {
                match rhs {
                    Self::Int(y) => Some(Self::Int(x + y)),
                    Self::Float(y) => Some(Self::Float(x as f64 + y)),
                    _ => None,
                }
            }
            Self::Float(x) => {
                match rhs {
                    Self::Int(y) => Some(Self::Float(x + y as f64)),
                    Self::Float(y) => Some(Self::Float(x + y)),
                    _ => None,
                }
            }
            Self::String(x) => {
                match rhs {
                    Self::String(y) => Some(Self::String(format!("{x}{y}"))),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

impl std::ops::Sub for Value {
    type Output = Option<Value>;

    fn sub(self, rhs: Self) -> Self::Output {
        match self {
            Self::Int(x) => {
                match rhs {
                    Self::Int(y) => Some(Self::Int(x - y)),
                    Self::Float(y) => Some(Self::Float(x as f64 - y)),
                    _ => None,
                }
            }
            Self::Float(x) => {
                match rhs {
                    Self::Int(y) => Some(Self::Float(x - y as f64)),
                    Self::Float(y) => Some(Self::Float(x - y)),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

impl std::ops::Mul for Value {
    type Output = Option<Value>;

    fn mul(self, rhs: Self) -> Self::Output {
        match self {
            Self::Int(x) => {
                match rhs {
                    Self::Int(y) => Some(Self::Int(x * y)),
                    Self::Float(y) => Some(Self::Float(x as f64 * y)),
                    _ => None,
                }
            }
            Self::Float(x) => {
                match rhs {
                    Self::Int(y) => Some(Self::Float(x * y as f64)),
                    Self::Float(y) => Some(Self::Float(x * y)),
                    _ => None,
                }
            }
            Self::String(x) => {
                match rhs {
                    Self::Int(y) => Some(Self::String(x.repeat(y as usize))),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

impl std::ops::Div for Value {
    type Output = Option<Value>;

    fn div(self, rhs: Self) -> Self::Output {
        match self {
            Self::Int(x) => {
                match rhs {
                    Self::Int(y) => Some(Self::Int(x / y)),
                    Self::Float(y) => Some(Self::Float(x as f64 / y)),
                    _ => None,
                }
            }
            Self::Float(x) => {
                match rhs {
                    Self::Int(y) => Some(Self::Float(x / y as f64)),
                    Self::Float(y) => Some(Self::Float(x / y)),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Float(x) => write!(f, "{x}"),
            Self::Int(x) => write!(f, "{x}"),
            Self::Bool(x) => write!(f, "{}", if *x { "true" } else { "false" }),
            Self::String(x) => write!(f, "{x}"),
            Self::Nil => write!(f, "nil"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FunctionDeclaration {
    name: String,
    r: Type,
    args: Vec<(String, Type)>,
}
