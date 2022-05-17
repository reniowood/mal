use std::collections::HashMap;

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
