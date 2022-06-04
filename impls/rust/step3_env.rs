use std::{cell::RefCell, collections::HashMap, rc::Rc};

use env::Env;
use printer::pr_str;
use reader::read_str;
use rustyline::Editor;
use types::{error, MalType};

mod env;
mod printer;
mod reader;
mod types;

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

    let mut env: Env = Env::new(None);
    env.set(
        "+".to_string(),
        MalType::Function(|args| binary_op(args, |a, b| a + b), None),
    );
    env.set(
        "-".to_string(),
        MalType::Function(|args| binary_op(args, |a, b| a - b), None),
    );
    env.set(
        "*".to_string(),
        MalType::Function(|args| binary_op(args, |a, b| a * b), None),
    );
    env.set(
        "/".to_string(),
        MalType::Function(|args| binary_op(args, |a, b| a / b), None),
    );
    let env = Rc::new(RefCell::new(env));

    loop {
        let line = rl.readline("user> ");
        match line {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                match rep(&line, &env) {
                    Ok(result) => println!("{}", result),
                    Err(message) => eprintln!("Error: {}", message),
                }
            }
            Err(_) => break,
        };
    }
}

fn rep(input: &str, env: &Rc<RefCell<Env>>) -> Result<String, String> {
    match read(input) {
        Ok(value) => eval(&value, env)
            .and_then(|result| Ok(print(&result)))
            .map_err(|err| print(&err)),
        Err(value) => Err(print(&value)),
    }
}

fn read(input: &str) -> Result<MalType, MalType> {
    read_str(input)
}

fn eval(ast: &MalType, env: &Rc<RefCell<Env>>) -> Result<MalType, MalType> {
    match ast {
        MalType::List(list, _) => {
            if list.is_empty() {
                return Ok(ast.clone());
            }

            match &list[0] {
                MalType::Symbol(name) if name == "def!" => {
                    let key = list[1].as_symbol()?;
                    let value = eval(&list[2], &env)?;
                    env.borrow_mut().set(key.clone(), value.clone());
                    Ok(value)
                }
                MalType::Symbol(name) if name == "let*" => {
                    let new_env = Rc::new(RefCell::new(Env::new(Some(env.clone()))));
                    let binding_list = list[1].as_list()?;
                    for i in (0..binding_list.len()).step_by(2) {
                        let key = binding_list[i].as_symbol()?;
                        let value = eval(&binding_list[i + 1], &new_env)?;
                        new_env.borrow_mut().set(key.clone(), value);
                    }
                    eval(&list[2], &new_env)
                }
                _ => {
                    let value = eval_ast(ast, &env)?;
                    let list = value.as_list()?;
                    list[0].as_function()?(&list[1..].to_vec())
                }
            }
        }
        _ => eval_ast(&ast, &env),
    }
}

fn eval_ast(ast: &MalType, env: &Rc<RefCell<Env>>) -> Result<MalType, MalType> {
    match ast {
        MalType::Symbol(name) => env
            .borrow()
            .get(name.as_str())
            .map(|value| value.clone())
            .ok_or(MalType::String(format!("'{}' not found", name))),
        MalType::List(list, metadata) => {
            let mut result = Vec::new();
            for value in list {
                result.push(eval(value, &env)?);
            }
            Ok(MalType::List(result, metadata.clone()))
        }
        MalType::Vector(list, metadata) => {
            let mut result = Vec::new();
            for value in list {
                result.push(eval(value, &env)?);
            }
            Ok(MalType::Vector(result, metadata.clone()))
        }
        MalType::Hashmap(map, metadata) => {
            let mut result = HashMap::new();
            for (key, value) in map {
                result.insert(key.clone(), eval(value, &env)?);
            }
            Ok(MalType::Hashmap(result, metadata.clone()))
        }
        _ => Ok(ast.clone()),
    }
}

fn print(ast: &MalType) -> String {
    pr_str(ast, true)
}
