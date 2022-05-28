use std::collections::HashMap;

use crate::printer::pr_str;
use crate::types::{Function, MalType};

pub fn ns() -> HashMap<&'static str, Function> {
    let mut ns: HashMap<&'static str, Function> = HashMap::new();
    ns.insert("+", |args| binary_number_op(args, |a, b| a + b));
    ns.insert("-", |args| binary_number_op(args, |a, b| a - b));
    ns.insert("*", |args| binary_number_op(args, |a, b| a * b));
    ns.insert("/", |args| binary_number_op(args, |a, b| a / b));
    ns.insert("prn", |args| {
        if args.is_empty() {
            println!();
        } else {
            println!("{}", join(args, true, " "));
        };
        Ok(MalType::Nil)
    });
    ns.insert("pr-str", |args| Ok(MalType::String(join(args, true, " "))));
    ns.insert("str", |args| Ok(MalType::String(join(args, false, ""))));
    ns.insert("println", |args| {
        println!("{}", join(args, false, " "));
        Ok(MalType::Nil)
    });
    ns.insert("list", |args| Ok(MalType::List(args.clone())));
    ns.insert("list?", |args| {
        unary_op(args, |v| {
            if let MalType::List(_) = v {
                Ok(MalType::True)
            } else {
                Ok(MalType::False)
            }
        })
    });
    ns.insert("empty?", |args| {
        unary_op(args, |v| match v {
            MalType::List(list) | MalType::Vector(list) => {
                if list.is_empty() {
                    Ok(MalType::True)
                } else {
                    Ok(MalType::False)
                }
            }
            value => Err(format!("Expected list but got {}.", value)),
        })
    });
    ns.insert("count", |args| {
        unary_op(args, |v| match v {
            MalType::List(list) | MalType::Vector(list) => Ok(MalType::Number(list.len() as i64)),
            MalType::Nil => Ok(MalType::Number(0)),
            value => Err(format!("Expected list or nil but got {}.", value)),
        })
    });
    ns.insert("=", |args| {
        binary_op(args, |a, b| {
            if a == b {
                Ok(MalType::True)
            } else {
                Ok(MalType::False)
            }
        })
    });
    ns.insert("<", |args| binary_boolean_op(args, |a, b| a < b));
    ns.insert("<=", |args| binary_boolean_op(args, |a, b| a <= b));
    ns.insert(">", |args| binary_boolean_op(args, |a, b| a > b));
    ns.insert(">=", |args| binary_boolean_op(args, |a, b| a >= b));
    ns
}

fn binary_number_op(args: &Vec<MalType>, op: fn(i64, i64) -> i64) -> Result<MalType, String> {
    match (&args[0], &args[1]) {
        (MalType::Number(a), MalType::Number(b)) => Ok(MalType::Number(op(*a, *b))),
        (MalType::Number(_), b) => Err(format!("Unexpected second argument {}.", b)),
        (a, MalType::Number(_)) => Err(format!("Unexpected first argument {}.", a)),
        (a, b) => Err(format!("Unexpected arguments {} and {}.", a, b)),
    }
}

fn binary_boolean_op(args: &Vec<MalType>, op: fn(i64, i64) -> bool) -> Result<MalType, String> {
    match (&args[0], &args[1]) {
        (MalType::Number(a), MalType::Number(b)) => Ok(if op(*a, *b) {
            MalType::True
        } else {
            MalType::False
        }),
        (MalType::Number(_), b) => Err(format!("Unexpected second argument {}.", b)),
        (a, MalType::Number(_)) => Err(format!("Unexpected first argument {}.", a)),
        (a, b) => Err(format!("Unexpected arguments {} and {}.", a, b)),
    }
}

fn binary_op(
    args: &Vec<MalType>,
    op: fn(&MalType, &MalType) -> Result<MalType, String>,
) -> Result<MalType, String> {
    op(&args[0], &args[1])
}

fn unary_op(
    args: &Vec<MalType>,
    op: fn(&MalType) -> Result<MalType, String>,
) -> Result<MalType, String> {
    op(&args[0])
}

fn join(v: &Vec<MalType>, print_readably: bool, separator: &str) -> String {
    v.iter()
        .map(|v| pr_str(v, print_readably))
        .collect::<Vec<String>>()
        .join(separator)
}
