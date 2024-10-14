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

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Float => "Float".into(),
            Self::Int => "Int".into(),
            Self::Bool => "Bool".into(),
            Self::String => "String".into(),
            Self::Nil => "Nil".into(),
            Self::Any => "Any".into(),
            Self::Function(r, _) => format!("Function -> {}", *r)
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Float(f64),
    Int(i64),
    Bool(bool),
    String(String),
    Nil,
}

impl Value {
    pub fn get_type(&self) -> Type {
        match self {
            Self::Float(_) => Type::Float,
            Self::Int(_) => Type::Int,
            Self::Bool(_) => Type::Bool,
            Self::String(_) => Type::String,
            Self::Nil => Type::Nil,
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
