use std::collections::HashMap;

use printer::pr_str;
use reader::read_str;
use rustyline::Editor;
use types::{error, Function, MalType};

mod env;
mod printer;
mod reader;
mod types;

type ReplEnv = HashMap<&'static str, Function>;

fn binary_op(args: &Vec<MalType>, op: fn(i64, i64) -> i64) -> Result<MalType, MalType> {
    match (&args[0], &args[1]) {
        (MalType::Number(a), MalType::Number(b)) => Ok(MalType::Number(op(*a, *b))),
        (MalType::Number(_), b) => error(format!("Unexpected second argument {}.", b)),
        (a, MalType::Number(_)) => error(format!("Unexpected first argument {}.", a)),
        (a, b) => error(format!("Unexpected arguments {} and {}.", a, b)),
    }
}

fn main() {
    let mut rl = Editor::<()>::new();

    let mut repl_env: ReplEnv = HashMap::new();
    repl_env.insert("+", |args| binary_op(args, |a, b| a + b));
    repl_env.insert("-", |args| binary_op(args, |a, b| a - b));
    repl_env.insert("*", |args| binary_op(args, |a, b| a * b));
    repl_env.insert("/", |args| binary_op(args, |a, b| a / b));

    loop {
        let line = rl.readline("user> ");
        match line {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                match rep(&line, &repl_env) {
                    Ok(result) => println!("{}", result),
                    Err(message) => eprintln!("Error: {}", message),
                }
            }
            Err(_) => break,
        };
    }
}

fn rep(input: &str, repl_env: &ReplEnv) -> Result<String, String> {
    match read(input) {
        Ok(value) => eval(&value, repl_env)
            .and_then(|result| Ok(print(&result)))
            .map_err(|err| print(&err)),
        Err(value) => Err(print(&value)),
    }
}

fn read(input: &str) -> Result<MalType, MalType> {
    read_str(input)
}

fn eval(ast: &MalType, repl_env: &ReplEnv) -> Result<MalType, MalType> {
    match ast {
        MalType::List(list, _) => {
            if list.is_empty() {
                return Ok(ast.clone());
            }

            let result = match eval_ast(ast, repl_env) {
                Ok(MalType::List(list, _)) => list,
                Ok(value) => return error(format!("Unexpected value {}.", value)),
                Err(message) => return Err(message),
            };

            match &result[0] {
                MalType::Function(f, _) => f(&result[1..].to_vec()),
                value => error(format!("Unexpected value {}.", pr_str(&value, true))),
            }
        }
        _ => eval_ast(&ast, repl_env),
    }
}

fn eval_ast(ast: &MalType, repl_env: &ReplEnv) -> Result<MalType, MalType> {
    match ast {
        MalType::Symbol(name) => repl_env
            .get(name.as_str())
            .ok_or(MalType::String(format!("Undefined symbol {}.", name)))
            .map(|f| MalType::Function(*f, None)),
        MalType::List(list, metadata) => {
            let mut result = Vec::new();
            for value in list {
                result.push(match eval(value, repl_env) {
                    Ok(value) => value,
                    Err(message) => return Err(message),
                });
            }
            Ok(MalType::List(result, metadata.clone()))
        }
        MalType::Vector(list, metadata) => {
            let mut result = Vec::new();
            for value in list {
                result.push(match eval(value, repl_env) {
                    Ok(value) => value,
                    Err(message) => return Err(message),
                });
            }
            Ok(MalType::Vector(result, metadata.clone()))
        }
        MalType::Hashmap(map, metadata) => {
            let mut result = HashMap::new();
            for (key, value) in map {
                result.insert(
                    key.clone(),
                    match eval(value, repl_env) {
                        Ok(value) => value,
                        Err(message) => return Err(message),
                    },
                );
            }
            Ok(MalType::Hashmap(result, metadata.clone()))
        }
        _ => Ok(ast.clone()),
    }
}

fn print(ast: &MalType) -> String {
    pr_str(ast, true)
}
