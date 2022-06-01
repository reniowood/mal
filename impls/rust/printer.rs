use std::fmt::Display;

use crate::types::{Hashable, MalType};

pub fn pr_str(value: &MalType, print_readably: bool) -> String {
    match value {
        MalType::True => "true".to_string(),
        MalType::False => "false".to_string(),
        MalType::Nil => "nil".to_string(),
        MalType::Number(number) => format!("{}", number),
        MalType::Keyword(name) => format!(":{}", name),
        MalType::Symbol(name) => format!("{}", name),
        MalType::String(value) => print_string(value, print_readably),
        MalType::List(list) => format!(
            "({})",
            list.iter()
                .map(|value| pr_str(value, print_readably))
                .collect::<Vec<String>>()
                .join(" ")
        ),
        MalType::Hashmap(value) => format!(
            "{{{}}}",
            value
                .iter()
                .map(|(key, value)| format!(
                    "{} {}",
                    print_hashable(key, print_readably),
                    pr_str(value, print_readably)
                ))
                .collect::<Vec<String>>()
                .join(" ")
        ),
        MalType::Vector(list) => format!(
            "[{}]",
            list.iter()
                .map(|value| pr_str(value, print_readably))
                .collect::<Vec<String>>()
                .join(" ")
        ),
        MalType::Deref(value) => format!("(deref {})", pr_str(value, print_readably)),
        MalType::WithMeta(first, second) => format!(
            "(with-meta {} {})",
            pr_str(second, print_readably),
            pr_str(first, print_readably)
        ),
        MalType::Function(_) => format!("#<function>"),
        MalType::Closure(_) => format!("#<function>"),
        MalType::Atom(v) => format!("(atom {})", pr_str(&v.borrow(), print_readably)),
    }
}

impl Display for MalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", pr_str(self, true))
    }
}

fn print_hashable(hashable: &Hashable, print_readably: bool) -> String {
    match hashable {
        Hashable::Keyword(name) => format!(":{}", name),
        Hashable::String(value) => print_string(value, print_readably),
    }
}

fn print_string(value: &String, print_readably: bool) -> String {
    if print_readably {
        format!("\"{}\"", escape_string(value))
    } else {
        value.to_string()
    }
}

fn escape_string(value: &str) -> String {
    value
        .chars()
        .map(|c| match c {
            '\\' => "\\\\".to_string(),
            '\n' => "\\n".to_string(),
            '\"' => "\\\"".to_string(),
            c => c.to_string(),
        })
        .collect()
}
