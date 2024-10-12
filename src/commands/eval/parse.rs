use std::error::Error;
use std::fmt::Display;
use super::tokenizer::{Token, Op};

#[derive(Debug)]
pub enum ParseTree {
    Leaf(Token),
    Branch(Op, Box<ParseTree>, Box<ParseTree>),
}

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedEndInput,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedEndInput => write!(f, "Input ended unexpectedly"),
        }
    }
}

impl Error for ParseError {}

impl ParseTree {
    pub fn new<'a, I: Iterator<Item = &'a Token>>(tokens: &mut I) -> Result<Self, ParseError> {
        if let Some(token) = tokens.next() {
            match token {
                Token::Scalar(_) | Token::Identifier(_) => Ok(Self::Leaf(token.clone())),
                Token::Operator(op) => {
                    let left = ParseTree::new(tokens)?;
                    let right = ParseTree::new(tokens)?;

                    Ok(Self::Branch(op.clone(), Box::new(left), Box::new(right)))
                }
            }
        } else {
            Err(ParseError::UnexpectedEndInput)
        }
    }

    pub fn evaluate(&self) -> f64 {
        match self {
            ParseTree::Leaf(Token::Scalar(value)) => *value,
            ParseTree::Leaf(Token::Identifier(_)) => unimplemented!(),
            ParseTree::Leaf(Token::Operator(_)) => panic!("This absolutely should not happen"),
            ParseTree::Branch(op, left, right) => match op {
                Op::Add => left.evaluate() + right.evaluate(),
                Op::Sub => left.evaluate() - right.evaluate(),
                Op::Mul => left.evaluate() * right.evaluate(),
                Op::Div => left.evaluate() / right.evaluate(),
                Op::Exp => left.evaluate().powf(right.evaluate()),
            }
        }
    }
}