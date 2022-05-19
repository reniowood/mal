use std::collections::HashMap;

use crate::printer::pr_str;

pub type Function = fn(&Vec<MalType>) -> Result<MalType, String>;

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
    Quote(Box<MalType>),
    QuasiQuote(Box<MalType>),
    Unquote(Box<MalType>),
    SpliceUnquote(Box<MalType>),
    Hashmap(HashMap<String, MalType>),
    Vector(Vec<MalType>),
    Deref(Box<MalType>),
    WithMeta(Box<MalType>, Box<MalType>),
    Function(Function),
}

impl MalType {
    pub fn as_symbol(&self) -> Result<&String, String> {
        match self {
            MalType::Symbol(name) => Ok(name),
            value => return Err(format!("Expected symbol but got {}.", pr_str(&value))),
        }
    }

    pub fn as_list(&self) -> Result<&Vec<MalType>, String> {
        match self {
            MalType::List(list) => Ok(list),
            MalType::Vector(list) => Ok(list),
            value => return Err(format!("Expected list but got {}.", pr_str(&value))),
        }
    }

    pub fn as_function(&self) -> Result<&Function, String> {
        match self {
            MalType::Function(f) => Ok(f),
            value => return Err(format!("Expected function but got {}.", pr_str(&value))),
        }
    }
}
