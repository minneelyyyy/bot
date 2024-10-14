use super::{Value, Type, FunctionDeclaration};
use super::parser::{ParseTree, ParseError};

use std::collections::HashMap;
use std::borrow::Cow;
use std::fmt::Display;
use std::error::Error;

#[derive(Debug)]
pub enum RuntimeError {
    ParseError(ParseError),
    NoOverloadForTypes,
    ImmutableError(String),
    VariableUndefined(String),
    FunctionUndeclared(String),
    FunctionUndefined(String),
    NotAVariable(String),
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError(e) => write!(f, "{e}"),
            Self::NoOverloadForTypes => write!(f, "No overload of this operator exists for these operands"),
            Self::ImmutableError(ident) => write!(f, "`{ident}` already exists and cannot be redefined"),
            Self::VariableUndefined(ident) => write!(f, "variable `{ident}` was not defined"),
            Self::FunctionUndeclared(ident) => write!(f, "function `{ident}` was not declared"),
            Self::FunctionUndefined(ident) => write!(f, "function `{ident}` was not defined"),
            Self::NotAVariable(ident) => write!(f, "`{ident}` is a function but was attempted to be used like a variable"),
        }
    }
}

impl Error for RuntimeError {}

#[derive(Clone)]
enum Evaluation {
    // at this point, it's type is set in stone
    Computed(Value),

    // at this point, it's type is unknown, and may contradict a variable's type
    // or not match the expected value of the expression, this is a runtime error
    Uncomputed(Box<ParseTree>),
}

#[derive(Clone)]
struct Function {
    decl: FunctionDeclaration,
    body: Option<Box<ParseTree>>,
}

#[derive(Clone)]
enum Object {
    Variable(Evaluation),
    Function(Function),
}

pub struct Executor<I: Iterator<Item = Result<ParseTree, ParseError>>> {
    exprs: I,
    globals: HashMap<String, Object>,
}

impl<I: Iterator<Item = Result<ParseTree, ParseError>>> Executor<I> {
    pub fn new(exprs: I) -> Self {
        Self {
            exprs,
            globals: HashMap::new(),
        }
    }

    fn exec(
        &mut self,
        tree: ParseTree,
        locals: &mut Cow<HashMap<String, Object>>,
        in_function: Option<&str>) -> Result<Value, RuntimeError>
    {
        match tree {
            ParseTree::Add(x, y) => (self.exec(*x, locals, in_function)? + self.exec(*y, locals, in_function)?)
                .ok_or(RuntimeError::NoOverloadForTypes),
            ParseTree::Sub(x, y) => (self.exec(*x, locals, in_function)? - self.exec(*y, locals, in_function)?)
                .ok_or(RuntimeError::NoOverloadForTypes),
            ParseTree::Mul(x, y) => (self.exec(*x, locals, in_function)? * self.exec(*y, locals, in_function)?)
                .ok_or(RuntimeError::NoOverloadForTypes),
            ParseTree::Div(x, y) => (self.exec(*x, locals, in_function)? / self.exec(*y, locals, in_function)?)
                .ok_or(RuntimeError::NoOverloadForTypes),
            ParseTree::Exp(x, y) => match (self.exec(*x, locals, in_function)?, self.exec(*y, locals, in_function)?) {
                (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x.pow(y as u32))),
                (Value::Int(x), Value::Float(y)) => Ok(Value::Float((x as f64).powf(y))),
                (Value::Float(x), Value::Int(y)) => Ok(Value::Float(x.powf(y as f64))),
                (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x.powf(y))),
                _ => Err(RuntimeError::NoOverloadForTypes),
            },
            ParseTree::EqualTo(x, y) => match (self.exec(*x, locals, in_function)?, self.exec(*y, locals, in_function)?) {
                (Value::Int(x), Value::Int(y)) => Ok(Value::Bool(x == y)),
                (Value::Int(x), Value::Float(y)) => Ok(Value::Bool(x as f64 == y)),
                (Value::Float(x), Value::Int(y)) => Ok(Value::Bool(x == y as f64)),
                (Value::Float(x), Value::Float(y)) => Ok(Value::Bool(x == y)),
                (Value::Bool(x), Value::Bool(y)) => Ok(Value::Bool(x == y)),
                (Value::String(x), Value::String(y)) => Ok(Value::Bool(x == y)),
                _ => Err(RuntimeError::NoOverloadForTypes)
            },
            ParseTree::GreaterThan(x, y) => match (self.exec(*x, locals, in_function)?, self.exec(*y, locals, in_function)?) {
                (Value::Int(x), Value::Int(y)) => Ok(Value::Bool(x > y)),
                (Value::Int(x), Value::Float(y)) => Ok(Value::Bool(x as f64 > y)),
                (Value::Float(x), Value::Int(y)) => Ok(Value::Bool(x > y as f64)),
                (Value::Float(x), Value::Float(y)) => Ok(Value::Bool(x > y)),
                _ => Err(RuntimeError::NoOverloadForTypes)
            },
            ParseTree::GreaterThanOrEqualTo(x, y) => match (self.exec(*x, locals, in_function)?, self.exec(*y, locals, in_function)?) {
                (Value::Int(x), Value::Int(y)) => Ok(Value::Bool(x >= y)),
                (Value::Int(x), Value::Float(y)) => Ok(Value::Bool(x as f64 >= y)),
                (Value::Float(x), Value::Int(y)) => Ok(Value::Bool(x >= y as f64)),
                (Value::Float(x), Value::Float(y)) => Ok(Value::Bool(x >= y)),
                _ => Err(RuntimeError::NoOverloadForTypes)
            },
            ParseTree::LessThan(x, y) => match (self.exec(*x, locals, in_function)?, self.exec(*y, locals, in_function)?) {
                (Value::Int(x), Value::Int(y)) => Ok(Value::Bool(x < y)),
                (Value::Int(x), Value::Float(y)) => Ok(Value::Bool((x as f64) < y)),
                (Value::Float(x), Value::Int(y)) => Ok(Value::Bool(x < y as f64)),
                (Value::Float(x), Value::Float(y)) => Ok(Value::Bool(x < y)),
                _ => Err(RuntimeError::NoOverloadForTypes)
            },
            ParseTree::LessThanOrEqualTo(x, y) => match (self.exec(*x, locals, in_function)?, self.exec(*y, locals, in_function)?) {
                (Value::Int(x), Value::Int(y)) => Ok(Value::Bool(x <= y)),
                (Value::Int(x), Value::Float(y)) => Ok(Value::Bool(x as f64 <= y)),
                (Value::Float(x), Value::Int(y)) => Ok(Value::Bool(x <= y as f64)),
                (Value::Float(x), Value::Float(y)) => Ok(Value::Bool(x <= y)),
                _ => Err(RuntimeError::NoOverloadForTypes)
            },
            ParseTree::Not(x) => match self.exec(*x, locals, in_function)? {
                Value::Bool(x) => Ok(Value::Bool(!x)),
                _ => Err(RuntimeError::NoOverloadForTypes)
            },
            ParseTree::Equ(ident, body, scope) => {
                if self.globals.contains_key(&ident) || locals.contains_key(&ident) {
                    Err(RuntimeError::ImmutableError(ident.clone()))
                } else {
                    let locals = locals.to_mut();
                    let value = self.exec(*body, &mut Cow::Borrowed(&locals), in_function)?;
                    locals.insert(ident.clone(), Object::Variable(Evaluation::Computed(value)));

                    self.exec(*scope, &mut Cow::Borrowed(&locals), in_function)
                }
            },
            ParseTree::LazyEqu(ident, body, scope) => {
                if self.globals.contains_key(&ident) || locals.contains_key(&ident) {
                    Err(RuntimeError::ImmutableError(ident.clone()))
                } else {
                    let locals = locals.to_mut();
                    locals.insert(ident.clone(), Object::Variable(Evaluation::Uncomputed(body)));

                    self.exec(*scope, &mut Cow::Borrowed(&locals), in_function)
                }
            },
            ParseTree::GlobalEqu(ident, body) => todo!(),
            ParseTree::LazyGlobalEqu(ident, body) => todo!(),
            ParseTree::FunctionDefinition(ident, args, r, body, scope) => {
                let existing = locals.get(&format!("{}{ident}",
                in_function.map(|s| format!("{s}:")).unwrap_or("".into())))
                    .or(locals.get(&ident).or(self.globals.get(&ident))).cloned();

                match existing {
                    Some(_) => Err(RuntimeError::ImmutableError(ident.clone())),
                    None => {
                        let locals = locals.to_mut();

                        locals.insert(ident.clone(), Object::Function(Function {
                            decl: FunctionDeclaration { name: ident.clone(), r, args },
                            body: Some(body)
                        }));

                        self.exec(*scope, &mut Cow::Borrowed(&locals), in_function)
                    }
                }
            },
            ParseTree::Compose(x, y) => {
                self.exec(*x, locals, in_function)?;
                self.exec(*y, locals, in_function)
            },
            ParseTree::Id(x) => self.exec(*x, locals, in_function),
            ParseTree::If(cond, body) => if match self.exec(*cond, locals, in_function)? {
                    Value::Float(f) => f != 0.0,
                    Value::Int(i) => i != 0,
                    Value::Bool(b) => b,
                    Value::String(s) => !s.is_empty(),
                    Value::Nil => false,
                } {
                    self.exec(*body, locals, in_function)
                } else {
                    Ok(Value::Nil)
                },
            ParseTree::IfElse(cond, istrue, isfalse) => if match self.exec(*cond, locals, in_function)? {
                Value::Float(f) => f != 0.0,
                Value::Int(i) => i != 0,
                Value::Bool(b) => b,
                Value::String(s) => !s.is_empty(),
                Value::Nil => false,
            } {
                self.exec(*istrue, locals, in_function)
            } else {
                self.exec(*isfalse, locals, in_function)
            },
            ParseTree::FunctionCall(ident, args) => {
                let obj = locals.get(&format!("{}{ident}", in_function.unwrap_or("")))
                    .or(locals.get(&ident)
                    .or(self.globals.get(&ident))).cloned();

                if let Some(Object::Function(f)) = obj {
                    let locals = locals.to_mut();
                    let body = f.body.ok_or(RuntimeError::FunctionUndefined(ident.clone()))?;

                    for ((name, _), tree) in std::iter::zip(f.decl.args, args) {
                        locals.insert(name.clone(), Object::Variable(Evaluation::Uncomputed(Box::new(tree))));
                    }

                    self.exec(*body, &mut Cow::Borrowed(&locals), Some(&ident))
                } else {
                    Err(RuntimeError::FunctionUndeclared(ident.clone()))
                }
            },
            ParseTree::Variable(ident) => {
                let locals = locals.to_mut();

                let obj = locals.get(&format!("{}{ident}",
                    in_function.map(|s| format!("{s}:")).unwrap_or("".into())))
                        .or(locals.get(&ident).or(self.globals.get(&ident))).cloned();

                if let Some(Object::Variable(eval)) = obj {
                    match eval {
                        Evaluation::Computed(v) => Ok(v),
                        Evaluation::Uncomputed(tree) => {
                            let v = self.exec(*tree, &mut Cow::Borrowed(&locals), in_function)?;
                            locals.insert(ident, Object::Variable(Evaluation::Computed(v.clone())));

                            Ok(v)
                        }
                    }
                } else {
                    Err(RuntimeError::VariableUndefined(ident.clone()))
                }
            },
            ParseTree::Constant(value) => Ok(value),
        }
    }
}

impl<I: Iterator<Item = Result<ParseTree, ParseError>>> Iterator for Executor<I> {
    type Item = Result<Value, RuntimeError>;

    fn next(&mut self) -> Option<Self::Item> {
        let expr = self.exprs.next();

        match expr {
            Some(Ok(expr)) => Some(self.exec(expr, &mut Cow::Borrowed(&HashMap::new()), None)),
            Some(Err(e)) => Some(Err(RuntimeError::ParseError(e))),
            None => None,
        }
    }
}
