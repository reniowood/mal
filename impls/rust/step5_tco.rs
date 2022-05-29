use std::{cell::RefCell, collections::HashMap, rc::Rc};

mod core;
mod env;
mod printer;
mod reader;
mod types;

use crate::core::ns;
use env::Env;
use printer::pr_str;
use reader::read_str;
use rustyline::Editor;
use types::{Closure, MalType};

fn main() {
    let mut rl = Editor::<()>::new();

    let mut env: Env = Env::new(None);
    for (symbol, function) in ns() {
        env.set(symbol.to_string(), MalType::Function(function));
    }
    let env = Rc::new(RefCell::new(env));

    let _ = rep("(def! not (fn* (a) (if a false true)))", env.clone());

    loop {
        let line = rl.readline("user> ");
        match line {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                match rep(&line, env.clone()) {
                    Ok(result) => println!("{}", result),
                    Err(message) => eprintln!("Error: {}", message),
                }
            }
            Err(_) => break,
        };
    }
}

fn rep(input: &str, env: Rc<RefCell<Env>>) -> Result<String, String> {
    match read(input) {
        Ok(value) => eval(&value, env).and_then(|result| Ok(print(&result))),
        Err(message) => Err(message),
    }
}

fn read(input: &str) -> Result<MalType, String> {
    read_str(input)
}

fn eval(ast: &MalType, env: Rc<RefCell<Env>>) -> Result<MalType, String> {
    let mut ast = ast.clone();
    let mut env = env;
    loop {
        match &ast {
            MalType::List(list) => {
                if list.is_empty() {
                    return Ok(ast.clone());
                }

                match &list[0] {
                    MalType::Symbol(name) if name == "def!" => {
                        let key = list[1].as_symbol()?;
                        let value = eval(&list[2], env.clone())?;
                        env.borrow_mut().set(key.clone(), value.clone());
                        return Ok(value);
                    }
                    MalType::Symbol(name) if name == "let*" => {
                        let new_env = Rc::new(RefCell::new(Env::new(Some(env.clone()))));
                        let binding_list = list[1].as_list()?;
                        for i in (0..binding_list.len()).step_by(2) {
                            let key = binding_list[i].as_symbol()?;
                            let value = eval(&binding_list[i + 1], new_env.clone())?;
                            new_env.borrow_mut().set(key.clone(), value);
                        }
                        env = new_env;
                        ast = list[2].clone();
                    }
                    MalType::Symbol(name) if name == "do" => {
                        let last_index = list.len() - 1;
                        let list = MalType::List(list[1..last_index].to_vec());
                        let result = eval_ast(&list, env.clone())?;
                        let result = result.as_list()?;
                        ast = result[last_index].clone();
                    }
                    MalType::Symbol(name) if name == "if" => {
                        let condition = eval(&list[1], env.clone())?;
                        match condition {
                            MalType::Nil | MalType::False => {
                                if list.len() > 3 {
                                    ast = list[3].clone();
                                } else {
                                    return Ok(MalType::Nil);
                                }
                            }
                            _ => ast = list[2].clone(),
                        };
                    }
                    MalType::Symbol(name) if name == "fn*" => {
                        let params = list[1].as_list()?;
                        let body = &list[2];
                        return Ok(MalType::Closure(Box::new(Closure::new(
                            params.clone(),
                            body.clone(),
                            env.clone(),
                        ))));
                    }
                    _ => {
                        let value = eval_ast(&ast, env.clone())?;
                        let list = value.as_list()?;
                        match &list[0] {
                            MalType::Closure(closure) => {
                                ast = closure.body.clone();
                                env = Rc::new(RefCell::new(Env::from(
                                    Some(closure.env.clone()),
                                    &closure.params,
                                    &list[1..].to_vec(),
                                )));
                            }
                            MalType::Function(function) => return function(&list[1..].to_vec()),
                            _ => return Err(format!("Expected function but got {}", &list[0])),
                        }
                    }
                };
            }
            _ => return eval_ast(&ast, env),
        };
    }
}

fn eval_ast(ast: &MalType, env: Rc<RefCell<Env>>) -> Result<MalType, String> {
    match ast {
        MalType::Symbol(name) => env
            .borrow()
            .get(name.as_str())
            .map(|value| value.clone())
            .ok_or(format!("'{}' not found.", name)),
        MalType::List(list) => {
            let mut result = Vec::new();
            for value in list {
                result.push(eval(value, env.clone())?);
            }
            Ok(MalType::List(result))
        }
        MalType::Vector(list) => {
            let mut result = Vec::new();
            for value in list {
                result.push(eval(value, env.clone())?);
            }
            Ok(MalType::Vector(result))
        }
        MalType::Hashmap(map) => {
            let mut result = HashMap::new();
            for (key, value) in map {
                result.insert(key.clone(), eval(value, env.clone())?);
            }
            Ok(MalType::Hashmap(result))
        }
        _ => Ok(ast.clone()),
    }
}

fn print(ast: &MalType) -> String {
    pr_str(ast, true)
}
