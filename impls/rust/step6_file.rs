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
use types::{error, Closure, MalType};

fn main() {
    let env = create_env();
    let args: Vec<String> = std::env::args().collect();

    load_utils(env.clone(), &args);

    if args.len() > 1 {
        run_file(&env, &args[1]);
    } else {
        repl(&env);
    }
}

fn create_env() -> Rc<RefCell<Env>> {
    let mut env: Env = Env::new(None);
    for (symbol, function) in ns() {
        env.set(symbol.to_string(), MalType::Function(function, None));
    }
    Rc::new(RefCell::new(env))
}

fn load_utils(env: Rc<RefCell<Env>>, args: &Vec<String>) {
    env.borrow_mut().set(
        "*ARGV*".to_string(),
        if args.len() < 3 {
            MalType::List(Vec::new(), None)
        } else {
            MalType::List(
                args[2..]
                    .iter()
                    .map(|v| MalType::String(v.to_string()))
                    .collect(),
                None,
            )
        },
    );
    let _ = rep("(def! not (fn* (a) (if a false true)))", &env);
    let _ = rep(
        r#"(def! load-file (fn* (f) (eval (read-string (str "(do " (slurp f) "\nnil)")))))"#,
        &env,
    );
}

fn run_file(env: &Rc<RefCell<Env>>, filename: &str) {
    let load_file = format!("(load-file \"{}\")", filename);
    if let Err(message) = rep(&load_file, &env) {
        eprintln!("Error: {}", message);
    }
}

fn repl(env: &Rc<RefCell<Env>>) {
    let mut rl = Editor::<()>::new();

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
    let mut ast = ast.clone();
    let mut env = env.clone();
    loop {
        match &ast {
            MalType::List(list, _) => {
                if list.is_empty() {
                    return Ok(ast.clone());
                }

                match &list[0] {
                    MalType::Symbol(name) if name == "def!" => {
                        let key = list[1].as_symbol()?;
                        let value = eval(&list[2], &env)?;
                        env.borrow_mut().set(key.clone(), value.clone());
                        return Ok(value);
                    }
                    MalType::Symbol(name) if name == "let*" => {
                        let new_env = Rc::new(RefCell::new(Env::new(Some(env.clone()))));
                        let binding_list = list[1].as_list()?;
                        for i in (0..binding_list.len()).step_by(2) {
                            let key = binding_list[i].as_symbol()?;
                            let value = eval(&binding_list[i + 1], &new_env)?;
                            new_env.borrow_mut().set(key.clone(), value);
                        }
                        env = new_env;
                        ast = list[2].clone();
                    }
                    MalType::Symbol(name) if name == "do" => {
                        match eval_ast(
                            &MalType::List(list[1..list.len() - 1].to_vec(), None),
                            &env,
                        )? {
                            MalType::List(_, _) => ast = list[list.len() - 1].clone(),
                            _ => return error("Invalid do expression".to_string()),
                        }
                    }
                    MalType::Symbol(name) if name == "if" => {
                        let condition = eval(&list[1], &env)?;
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
                        return Ok(MalType::Closure(
                            Box::new(Closure::new(
                                params.clone(),
                                body.clone(),
                                env.clone(),
                                |env, params, args, body| {
                                    eval(
                                        body,
                                        &Rc::new(RefCell::new(Env::from(
                                            Some(env.clone()),
                                            &params,
                                            &args,
                                        ))),
                                    )
                                },
                            )),
                            None,
                        ));
                    }
                    MalType::Symbol(name) if name == "eval" => {
                        ast = eval(&list[1], &env)?;
                        if let Some(outer) = &env.clone().borrow().outer {
                            env = outer.clone();
                        }
                    }
                    _ => {
                        let value = eval_ast(&ast, &env)?;
                        let list = value.as_list()?;
                        match &list[0] {
                            MalType::Closure(closure, _) => {
                                ast = closure.body.clone();
                                env = Rc::new(RefCell::new(Env::from(
                                    Some(closure.env.clone()),
                                    &closure.params,
                                    &list[1..].to_vec(),
                                )));
                            }
                            MalType::Function(function, _) => return function(&list[1..].to_vec()),
                            _ => return error(format!("Expected function but got {}", &list[0])),
                        }
                    }
                };
            }
            _ => return eval_ast(&ast, &env),
        };
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
