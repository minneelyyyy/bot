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

    pub fn add() -> Token {
        Token::Operator(Op::Add)
    }

    pub fn sub() -> Token {
        Token::Operator(Op::Sub)
    }

    pub fn mul() -> Token {
        Token::Operator(Op::Mul)
    }

    pub fn div() -> Token {
        Token::Operator(Op::Div)
    }

    pub fn exp() -> Token {
        Token::Operator(Op::Exp)
    }

    pub fn tokenize(s: &str) -> Result<Vec<Self>, Error> {
        s.split_whitespace().map(Token::from_str).collect()
    }
}

impl FromStr for Token {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            // First check if s is an operator
            "+"  => Ok(Token::add()),
            "-"  => Ok(Token::sub()),
            "*"  => Ok(Token::mul()),
            "**" => Ok(Token::exp()),
            "/"  => Ok(Token::div()),
            _ => {
                if s.starts_with(|c| char::is_digit(c, 10)) {
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