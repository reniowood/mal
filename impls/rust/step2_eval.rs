use std::collections::HashMap;

use printer::pr_str;
use reader::read_str;
use rustyline::Editor;
use types::{MalType, Function};

mod reader;
mod types;
mod printer;

type ReplEnv = HashMap<&'static str, Function>;

fn binary_op(args: &Vec<MalType>, op: fn(i64, i64) -> i64) -> Result<MalType, String> {
    match (&args[0], &args[1]) {
        (MalType::Number(a), MalType::Number(b)) => Ok(MalType::Number(op(*a, *b))),
        (MalType::Number(_), b) => Err(format!("Unexpected second argument {}.", b)),
        (a, MalType::Number(_)) => Err(format!("Unexpected first argument {}.", a)),
        (a, b) => Err(format!("Unexpected arguments {} and {}.", a, b)),
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
            Err(_) => break
        };
    }
}

fn rep(input: &str, repl_env: &ReplEnv) -> Result<String, String> {
    match read(input) {
        Ok(value) => eval(&value, repl_env).and_then(|result| Ok(print(&result))),
        Err(message) => Err(message)
    }
}

fn read(input: &str) -> Result<MalType, String> {
    read_str(input)
}

fn eval(ast: &MalType, repl_env: &ReplEnv) -> Result<MalType, String> {
    match ast {
        MalType::List(list) => {
            if list.is_empty() {
                return Ok(ast.clone());
            }

            let result = match eval_ast(ast, repl_env) {
                Ok(MalType::List(list)) => list,
                Ok(value) => return Err(format!("Unexpected value {}.", value)),
                Err(message) => return Err(message),
            };

            match &result[0] {
                MalType::Function(f) => f(&result[1..].to_vec()),
                value => Err(format!("Unexpected value {}.", pr_str(&value))),
            }
        }
        _ => eval_ast(&ast, repl_env)
    }
}

fn eval_ast(ast: &MalType, repl_env: &ReplEnv) -> Result<MalType, String> {
    match ast {
        MalType::Symbol(name) => repl_env.get(name.as_str()).ok_or(format!("Undefined symbol {}.", name)).map(|f| MalType::Function(*f)),
        MalType::List(list) => {
            let mut result = Vec::new();
            for value in list {
                result.push(match eval(value, repl_env) {
                    Ok(value) => value,
                    Err(message) => return Err(message),
                });
            }
            Ok(MalType::List(result))
        }
        MalType::Vector(list) => {
            let mut result = Vec::new();
            for value in list {
                result.push(match eval(value, repl_env) {
                    Ok(value) => value,
                    Err(message) => return Err(message),
                });
            }
            Ok(MalType::Vector(result))
        }
        MalType::Hashmap(map) => {
            let mut result = HashMap::new();
            for (key, value) in map {
                result.insert(key.clone(), match eval(value, repl_env) {
                    Ok(value) => value,
                    Err(message) => return Err(message),
                });
            }
            Ok(MalType::Hashmap(result))
        }
        _ => Ok(ast.clone()),
    }
}

fn print(ast: &MalType) -> String {
    pr_str(ast)
}
