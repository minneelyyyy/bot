use std::error;

use crate::common::Error;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Exp,
    Equ,
    LazyEqu,
    GlobalEqu,
    LazyGlobalEqu,
    FunctionDeclare(usize),
    Compose,
    Id,
}

#[derive(Debug, Clone)]
pub enum Token {
    Identifier(String),
    Scalar(f64),
    Operator(Op),
}

#[derive(Debug, Clone)]
struct ParseError(String);

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl error::Error for ParseError {
    fn description(&self) -> &str {
        &self.0
    }
}

impl Token {
    pub fn identifier(str: String) -> Token {
        Token::Identifier(str)
    }

    pub fn scalar(value: f64) -> Token {
        Token::Scalar(value)
    }

    pub fn operator(op: Op) -> Token {
        Token::Operator(op)
    }
    pub fn tokenize(s: &str) -> Result<Vec<Self>, Error> {
        s.split_whitespace().map(Token::from_str).collect()
    }
}

fn get_dot_count<I: Iterator<Item = char>>(s: I) -> usize {
    s.fold(0, |acc, c| acc + match c {
        ':' => 2,
        '.' => 1,
        _ => 0,
    })
}

impl FromStr for Token {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = if s.starts_with("\\") { s.chars().skip(1).collect() } else { s.to_string() };

        match s.as_str() {
            // First check if s is an operator
            "+"  => Ok(Token::operator(Op::Add)),
            "-"  => Ok(Token::operator(Op::Sub)),
            "*"  => Ok(Token::operator(Op::Mul)),
            "**" => Ok(Token::operator(Op::Exp)),
            "/"  => Ok(Token::operator(Op::Div)),
            "="  => Ok(Token::operator(Op::Equ)),
            "."  => Ok(Token::operator(Op::LazyEqu)),
            "=>" => Ok(Token::operator(Op::GlobalEqu)),
            ".>" => Ok(Token::operator(Op::LazyGlobalEqu)),
            "~"  => Ok(Token::operator(Op::Compose)),
            "," => Ok(Token::operator(Op::Id)),
            _ => {
                // variable length operators
                if s.starts_with(':') {
                    Ok(Token::operator(Op::FunctionDeclare(1 + get_dot_count(s[1..].chars()))))
                } else if s.starts_with(|c| char::is_digit(c, 10)) {
                    Ok(Token::scalar(s.parse()?))
                } else if s.starts_with(char::is_alphabetic)
                        && s.chars().skip(1).all(char::is_alphanumeric) {
                    Ok(Token::identifier(s.to_string()))
                } else {
                    Err(Box::new(ParseError(format!("Failed to parse \"{}\"", s))))
                }
            }
        }
    }
}