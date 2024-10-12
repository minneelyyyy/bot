use std::error::Error;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Display;
use crate::common;
use super::tokenizer::{self, Token};

#[derive(Clone, Debug)]
pub enum ParseTree<'a> {
    Add(Box<ParseTree<'a>>, Box<ParseTree<'a>>),
    Sub(Box<ParseTree<'a>>, Box<ParseTree<'a>>),
    Mul(Box<ParseTree<'a>>, Box<ParseTree<'a>>),
    Div(Box<ParseTree<'a>>, Box<ParseTree<'a>>),
    Exp(Box<ParseTree<'a>>, Box<ParseTree<'a>>),
    Equ(&'a str, Box<ParseTree<'a>>, Box<ParseTree<'a>>),
    GlobalEqu(&'a str, Box<ParseTree<'a>>),
    Compose(Box<ParseTree<'a>>, Box<ParseTree<'a>>),
    Id(Box<ParseTree<'a>>),
    FunctionDeclaration(&'a str, Vec<Object<'a>>, Box<ParseTree<'a>>, Box<ParseTree<'a>>),
    FunctionApplication(&'a str, Vec<ParseTree<'a>>),
    Variable(&'a str),
    Scalar(f64),
}

#[derive(Clone, Debug)]
pub struct FunctionDeclaration<'a> {
    name: &'a str,
    args: Vec<Object<'a>>,
}

#[derive(Clone, Debug)]
pub struct Function<'a> {
    decl: FunctionDeclaration<'a>,
    body: Option<Box<ParseTree<'a>>>, // may be used in declarations where a value isn't provided
}

#[derive(Clone, Debug)]
pub struct Variable<'a> {
    name: &'a str,
    body: Option<Box<ParseTree<'a>>>, // may be used in declarations where a value isn't provided
}

impl<'a> Variable<'a> {
    pub fn new(name: &'a str, body: Option<Box<ParseTree<'a>>>) -> Self {
        Self { name, body }
    }
}

#[derive(Clone, Debug)]
pub enum Object<'a> {
    Variable(Variable<'a>),
    Func(Function<'a>),
}

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedEndInput,
    IdentifierUndefined(String),
    InvalidIdentifier,
    FunctionUndefined,
    VariableUndefined,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedEndInput => write!(f, "Input ended unexpectedly"),
            ParseError::IdentifierUndefined(name) => write!(f, "Undefined variable `{}`", name),
            ParseError::InvalidIdentifier => write!(f, "Invalid identifier"),
            ParseError::FunctionUndefined => write!(f, "Undefined function"),
            ParseError::VariableUndefined => write!(f, "Undefined variable"),
        }
    }
}

impl Error for ParseError {}

impl<'a> ParseTree<'a> {
    pub fn parse<I: Iterator<Item = &'a Token>>(
        tokens: &mut I,
        globals: &HashMap<String, FunctionDeclaration<'a>>,
        locals: &mut Cow<HashMap<String, FunctionDeclaration<'a>>>) -> Result<Self, ParseError>
    {
        if let Some(token) = tokens.next() {
            match token {
                // Just return scalars
                Token::Scalar(x) => Ok(ParseTree::Scalar(*x)),

                // Get any identifiers as objects from local first then global scope
                Token::Identifier(ident) => {
                    // If it is found to be a function, get its argument count.
                    // During parsing, we only keep track of function definitions
                    // so that we know how many arguments it takes
                    if let Some(decl) = locals.clone().get(ident).or(globals.get(ident)) {
                        let args = decl.args.iter()
                            .map(|_| ParseTree::parse(tokens, globals, locals)).collect::<Result<Vec<_>, ParseError>>()?;

                        Ok(ParseTree::FunctionApplication(ident, args))
                    } else {
                        Ok(ParseTree::Variable(ident))
                    }
                }

                Token::Operator(op) => {
                    match op {
                        tokenizer::Op::Add => Ok(ParseTree::Add(
                            Box::new(ParseTree::parse(tokens, globals, locals)?),
                            Box::new(ParseTree::parse(tokens, globals, locals)?)
                        )),
                        tokenizer::Op::Sub => Ok(ParseTree::Sub(
                            Box::new(ParseTree::parse(tokens, globals, locals)?),
                            Box::new(ParseTree::parse(tokens, globals, locals)?)
                        )),
                        tokenizer::Op::Mul => Ok(ParseTree::Mul(
                            Box::new(ParseTree::parse(tokens, globals, locals)?),
                            Box::new(ParseTree::parse(tokens, globals, locals)?)
                        )),
                        tokenizer::Op::Div => Ok(ParseTree::Div(
                            Box::new(ParseTree::parse(tokens, globals, locals)?),
                            Box::new(ParseTree::parse(tokens, globals, locals)?)
                        )),
                        tokenizer::Op::Exp => Ok(ParseTree::Exp(
                            Box::new(ParseTree::parse(tokens, globals, locals)?),
                            Box::new(ParseTree::parse(tokens, globals, locals)?)
                        )),
                        tokenizer::Op::Equ | tokenizer::Op::LazyEqu => {
                            let token = tokens.next().ok_or(ParseError::UnexpectedEndInput)?;

                            if let Token::Identifier(ident) = token {
                                Ok(ParseTree::Equ(
                                    ident,
                                    Box::new(ParseTree::parse(tokens, globals, locals)?),
                                    Box::new(ParseTree::parse(tokens, globals, locals)?)))
                            } else {
                                Err(ParseError::InvalidIdentifier)
                            }
                        }
                        tokenizer::Op::GlobalEqu | tokenizer::Op::LazyGlobalEqu => {
                            let token = tokens.next().ok_or(ParseError::UnexpectedEndInput)?;

                            if let Token::Identifier(ident) = token {
                                Ok(ParseTree::GlobalEqu(
                                    ident,
                                    Box::new(ParseTree::parse(tokens, globals, locals)?)
                                ))
                            } else {
                                Err(ParseError::InvalidIdentifier)
                            }
                        }
                        tokenizer::Op::FunctionDeclare(arg_count) => {
                            let token = tokens.next().ok_or(ParseError::UnexpectedEndInput)?;
                            
                            if let Token::Identifier(ident) = token {
                                let args: Vec<Object> = tokens.take(*arg_count)
                                    .map(|token| match token {
                                        Token::Identifier(s)
                                            => Ok(Object::Variable(Variable::new(s, None))),
                                        _ => Err(ParseError::InvalidIdentifier),
                                    }).collect::<Result<_, ParseError>>()?;

                                if args.len() < *arg_count {
                                    return Err(ParseError::InvalidIdentifier);
                                }

                                let locals = locals.to_mut();

                                locals.insert(ident.clone(), FunctionDeclaration {
                                    name: ident,
                                    args: args.clone()
                                });

                                Ok(ParseTree::FunctionDeclaration(
                                    ident,
                                    args,
                                    Box::new(ParseTree::parse(tokens, globals, &mut Cow::Borrowed(&*locals))?),
                                    Box::new(ParseTree::parse(tokens, globals, &mut Cow::Borrowed(&*locals))?)))
                            } else {
                                Err(ParseError::InvalidIdentifier)
                            }
                        }
                        tokenizer::Op::Compose => {
                            Ok(ParseTree::Compose(
                                Box::new(ParseTree::parse(tokens, globals, locals)?),
                                Box::new(ParseTree::parse(tokens, globals, locals)?)
                            ))
                        }
                        tokenizer::Op::Id =>
                            Ok(ParseTree::Id(Box::new(ParseTree::parse(tokens, globals, locals)?)))
                    }
                }
            }
        } else {
            Err(ParseError::UnexpectedEndInput)
        }
    }

    pub fn evaluate(
        self,
        globals: &mut HashMap<String, Object<'a>>,
        locals: &mut Cow<HashMap<String, Object<'a>>>) -> Result<f64, common::Error>
    {
        match self {
            ParseTree::Add(l, r) => Ok(l.evaluate(globals, locals)? + r.evaluate(globals, locals)?),
            ParseTree::Sub(l, r) => Ok(l.evaluate(globals, locals)? - r.evaluate(globals, locals)?),
            ParseTree::Mul(l, r) => Ok(l.evaluate(globals, locals)? * r.evaluate(globals, locals)?),
            ParseTree::Div(l, r) => Ok(l.evaluate(globals, locals)? / r.evaluate(globals, locals)?),
            ParseTree::Exp(l, r)
                => Ok(l.evaluate(globals, locals)?.powf(r.evaluate(globals, locals)?)),
            ParseTree::Equ(ident, value, body) => {
                let value = value.evaluate(globals, locals)?;

                let locals = locals.to_mut();

                locals.insert(ident.to_string(),
                              Object::Variable(
                                  Variable::new(ident, Some(Box::new(ParseTree::Scalar(value))))));

                body.evaluate(globals, &mut Cow::Borrowed(&locals))
            }
            ParseTree::GlobalEqu(ident, body) => {
                globals.insert(ident.to_string(),
                               Object::Variable(Variable::new(ident, Some(body.clone()))));

                Ok(0.0)
            }
            ParseTree::Compose(l, r) => {
                let _ = l.evaluate(globals, locals);
                r.evaluate(globals, locals)
            }
            ParseTree::Id(body) => body.evaluate(globals, locals),
            ParseTree::FunctionDeclaration(name, args, body, cont) => {
                let locals = locals.to_mut();

                locals.insert(name.to_string(), Object::Func(Function {
                    decl: FunctionDeclaration {
                        name,
                        args: args.clone(),
                    },
                    body: Some(body.clone())
                }));

                cont.evaluate(globals, &mut Cow::Borrowed(&locals))
            }
            ParseTree::FunctionApplication(name, params) => {
                let locals = locals.to_mut();
                let obj = locals.get(name).or(globals.get(name)).cloned();

                if let Some(Object::Func(func)) = obj {
                    for (param, arg) in params.iter().zip(func.decl.args.iter()) {
                        match arg {
                            Object::Variable(v)
                                => locals.insert(
                                    v.name.to_string(),
                                    Object::Variable(
                                        Variable::new(
                                            &v.name,
                                            Some(Box::new(
                                                ParseTree::Scalar(
                                                    param.clone().evaluate(
                                                        globals, &mut Cow::Borrowed(&locals))?)))))),
                            Object::Func(func)
                                => locals.insert(
                                    func.decl.name.to_string(),
                                    Object::Func(func.clone()))
                        };
                    }

                    func.body.ok_or(ParseError::FunctionUndefined.into())
                        .and_then(|body|
                            body.clone().evaluate(globals, &mut Cow::Borrowed(&locals)))
                } else {
                    Err(ParseError::FunctionUndefined.into())
                }
            }
            ParseTree::Variable(ident) => {
                let locals = locals.to_mut();
                let obj = locals.get(ident).or(globals.get(ident)).cloned();

                if let Some(Object::Variable(obj)) = obj {
                    return obj.body.clone().ok_or(ParseError::VariableUndefined.into())
                        .and_then(|body| body.clone().evaluate(globals, &mut Cow::Borrowed(&locals)));
                }

                Err(ParseError::IdentifierUndefined(ident.to_string()).into())
            }
            ParseTree::Scalar(x) => Ok(x),
        }
    }
}
