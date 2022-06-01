use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

use crate::{env::Env, printer::pr_str};

pub type Function = fn(&Vec<MalType>) -> Result<MalType, String>;
pub type ClosureFunction =
    fn(Rc<RefCell<Env>>, &Vec<MalType>, &Vec<MalType>, &MalType) -> Result<MalType, String>;

#[derive(Clone)]
pub struct Closure {
    pub params: Vec<MalType>,
    pub body: MalType,
    pub env: Rc<RefCell<Env>>,
    pub f: ClosureFunction,
}

impl Closure {
    pub fn new(
        params: Vec<MalType>,
        body: MalType,
        env: Rc<RefCell<Env>>,
        f: ClosureFunction,
    ) -> Self {
        Closure {
            params,
            body,
            env,
            f,
        }
    }

    pub fn apply(&self, args: &Vec<MalType>) -> Result<MalType, String> {
        (self.f)(self.env.clone(), &self.params, args, &self.body)
    }
}

impl Debug for Closure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Closure")
            .field("params", &self.params)
            .field("body", &self.body)
            .field("env", &self.env)
            .finish()
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum Hashable {
    Keyword(String),
    String(String),
}

#[derive(Clone)]
pub enum MalType {
    True,
    False,
    Nil,
    Number(i64),
    Symbol(String),
    Keyword(String),
    String(String),
    List(Vec<MalType>),
    Hashmap(HashMap<Hashable, MalType>),
    Vector(Vec<MalType>),
    Deref(Box<MalType>),
    WithMeta(Box<MalType>, Box<MalType>),
    Function(Function),
    Closure(Box<Closure>),
    Atom(Rc<RefCell<MalType>>),
}

impl MalType {
    pub fn as_symbol(&self) -> Result<&String, String> {
        match self {
            MalType::Symbol(name) => Ok(name),
            value => return Err(format!("Expected symbol but got {}.", pr_str(&value, true))),
        }
    }

    pub fn as_string(&self) -> Result<&String, String> {
        match self {
            MalType::String(value) => Ok(value),
            value => return Err(format!("Expected string but got {}.", pr_str(&value, true))),
        }
    }

    pub fn as_list(&self) -> Result<&Vec<MalType>, String> {
        match self {
            MalType::List(list) => Ok(list),
            MalType::Vector(list) => Ok(list),
            value => return Err(format!("Expected list but got {}.", pr_str(&value, true))),
        }
    }

    pub fn as_function(&self) -> Result<&Function, String> {
        match self {
            MalType::Function(f) => Ok(f),
            value => {
                return Err(format!(
                    "Expected function but got {}.",
                    pr_str(&value, true)
                ))
            }
        }
    }

    pub fn symbol(name: &str) -> Self {
        MalType::Symbol(name.to_string())
    }
}

impl PartialEq for MalType {
    fn eq(&self, other: &MalType) -> bool {
        match (self, other) {
            (MalType::True, MalType::True) => true,
            (MalType::False, MalType::False) => true,
            (MalType::Nil, MalType::Nil) => true,
            (MalType::Number(a), MalType::Number(b)) => a == b,
            (MalType::Symbol(a), MalType::Symbol(b)) => a == b,
            (MalType::Keyword(a), MalType::Keyword(b)) => a == b,
            (MalType::String(a), MalType::String(b)) => a == b,
            (MalType::List(a), MalType::List(b)) => a == b,
            (MalType::Hashmap(a), MalType::Hashmap(b)) => a == b,
            (MalType::Vector(a), MalType::Vector(b)) => a == b,
            (MalType::List(a), MalType::Vector(b)) => a == b,
            (MalType::Vector(a), MalType::List(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for MalType {}

impl Debug for MalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::True => write!(f, "True"),
            Self::False => write!(f, "False"),
            Self::Nil => write!(f, "Nil"),
            Self::Number(arg0) => f.debug_tuple("Number").field(arg0).finish(),
            Self::Symbol(arg0) => f.debug_tuple("Symbol").field(arg0).finish(),
            Self::Keyword(arg0) => f.debug_tuple("Keyword").field(arg0).finish(),
            Self::String(arg0) => f.debug_tuple("String").field(arg0).finish(),
            Self::List(arg0) => f.debug_tuple("List").field(arg0).finish(),
            Self::Hashmap(arg0) => f.debug_tuple("Hashmap").field(arg0).finish(),
            Self::Vector(arg0) => f.debug_tuple("Vector").field(arg0).finish(),
            Self::Deref(arg0) => f.debug_tuple("Deref").field(arg0).finish(),
            Self::WithMeta(arg0, arg1) => {
                f.debug_tuple("WithMeta").field(arg0).field(arg1).finish()
            }
            Self::Function(_) => f.debug_tuple("Function").finish(),
            Self::Closure(_) => f.debug_tuple("Closure").finish(),
            Self::Atom(arg0) => f.debug_tuple("Atom").field(arg0).finish(),
        }
    }
}
