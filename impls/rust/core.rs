use crate::types::Hashable;
use crate::Rc;
use crate::RefCell;
use std::collections::HashMap;
use std::fs;

use crate::printer::pr_str;
use crate::reader::read_str;
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
    ns.insert("read-string", |args| {
        args[0].as_string().and_then(|v| read_str(v))
    });
    ns.insert("slurp", |args| {
        args[0].as_string().and_then(|v| read_file(v))
    });
    ns.insert("atom", |args| {
        Ok(MalType::Atom(Rc::new(RefCell::new(args[0].clone()))))
    });
    ns.insert("atom?", |args| {
        Ok(if let MalType::Atom(_) = args[0] {
            MalType::True
        } else {
            MalType::False
        })
    });
    ns.insert("deref", |args| match &args[0] {
        MalType::Atom(v) => Ok(v.borrow().clone()),
        v => Err(format!("Expected atom, but got {}", v)),
    });
    ns.insert("reset!", |args| {
        if let MalType::Atom(v) = &args[0] {
            *v.borrow_mut() = args[1].clone();
            Ok(v.borrow().clone())
        } else {
            Err(format!("Expected atom, but got {}", &args[0]))
        }
    });
    ns.insert("swap!", |args| {
        let atom_value = if let MalType::Atom(v) = &args[0] {
            v
        } else {
            return Err(format!("Expected atom, but got {}", &args[0]));
        };

        let mut f_args = Vec::new();
        f_args.push(atom_value.borrow().clone());
        if args.len() > 2 {
            f_args.extend(args[2..].to_vec());
        }

        let result = match &args[1] {
            MalType::Function(f) => match f(&f_args) {
                Ok(MalType::Exception(e)) => return Ok(MalType::Exception(e)),
                result => result,
            },
            MalType::Closure(closure) => closure.apply(&f_args),
            _ => return Err(format!("Expected function, but got {}", &args[1])),
        };

        if let Ok(value) = &result {
            *atom_value.borrow_mut() = value.clone();
        }

        result
    });
    ns.insert("cons", |args| {
        let head = &args[0];
        let tail = match &args[1] {
            MalType::List(list) | MalType::Vector(list) => list,
            _ => return Err(format!("Expected list or vector, but got {}", &args[1])),
        };

        let mut list = Vec::new();
        list.push(head.clone());
        list.extend(tail.clone());

        Ok(MalType::List(list))
    });
    ns.insert("concat", |args| {
        let mut result = Vec::new();
        for arg in args {
            match arg {
                MalType::List(list) | MalType::Vector(list) => result.extend(list.clone()),
                _ => return Err(format!("Expected list or vector, but got {}", arg)),
            };
        }

        Ok(MalType::List(result))
    });
    ns.insert("vec", |args| match &args[0] {
        MalType::List(list) => Ok(MalType::Vector(list.clone())),
        MalType::Vector(_) => Ok(args[0].clone()),
        _ => Err(format!("Expected list or vector, but got {}", &args[0])),
    });
    ns.insert("nth", |args| {
        let index = match &args[1] {
            MalType::Number(value) => *value as usize,
            _ => return Err(format!("Expected number, but got {}", &args[1])),
        };
        match &args[0] {
            MalType::List(list) | MalType::Vector(list) if list.len() <= index => Err(format!(
                "Out of range: The index was {} but the size of the list is {}",
                index,
                list.len()
            )),
            MalType::List(list) | MalType::Vector(list) => Ok(list.get(index).unwrap().clone()),
            _ => Err(format!("Expected list or vector, but got {}", &args[0])),
        }
    });
    ns.insert("first", |args| match &args[0] {
        MalType::Nil => Ok(MalType::Nil),
        MalType::List(list) | MalType::Vector(list) if list.is_empty() => Ok(MalType::Nil),
        MalType::List(list) | MalType::Vector(list) => Ok(list.get(0).unwrap().clone()),
        _ => Err(format!("Expected list or vector, but got {}", &args[0])),
    });
    ns.insert("rest", |args| match &args[0] {
        MalType::Nil => Ok(MalType::List(vec![])),
        MalType::List(list) | MalType::Vector(list) if list.is_empty() => Ok(MalType::List(vec![])),
        MalType::List(list) | MalType::Vector(list) => Ok(MalType::List(list[1..].to_vec())),
        _ => Err(format!("Expected list or vector, but got {}", &args[0])),
    });
    ns.insert("throw", |args| {
        Ok(MalType::Exception(Box::new(args[0].clone())))
    });
    ns.insert("apply", |args| {
        let last_index = args.len() - 1;
        let f_args = match &args[last_index] {
            MalType::List(list) | MalType::Vector(list) => {
                let mut f_args = Vec::new();
                f_args.extend(args[1..last_index].to_vec());
                f_args.extend(list.clone());
                f_args
            }
            last_arg => return Err(format!("Expected list or vector, but got {}", &last_arg)),
        };
        match &args[0] {
            MalType::Closure(closure) => closure.apply(&f_args),
            MalType::Function(f) => match f(&f_args) {
                Ok(MalType::Exception(e)) => return Ok(MalType::Exception(e)),
                result => result,
            },
            _ => Err(format!("Expected function, but got {}", &args[0])),
        }
    });
    ns.insert("map", |args| match &args[1] {
        MalType::List(list) | MalType::Vector(list) => {
            let mut result = Vec::new();
            for value in list {
                let f_args = vec![value.clone()];
                let value = match &args[0] {
                    MalType::Closure(closure) => closure.apply(&f_args),
                    MalType::Function(f) => match f(&f_args) {
                        Ok(MalType::Exception(e)) => return Ok(MalType::Exception(e)),
                        result => result,
                    },
                    _ => return Err(format!("Expected function, but got {}", &args[0])),
                };
                match value {
                    Ok(value) => result.push(value),
                    Err(message) => return Err(message),
                };
            }
            Ok(MalType::List(result))
        }
        _ => Err(format!("Expected list or vector, but got {}", &args[0])),
    });
    ns.insert("nil?", |args| {
        Ok(if let MalType::Nil = &args[0] {
            MalType::True
        } else {
            MalType::False
        })
    });
    ns.insert("true?", |args| {
        Ok(if let MalType::True = &args[0] {
            MalType::True
        } else {
            MalType::False
        })
    });
    ns.insert("false?", |args| {
        Ok(if let MalType::False = &args[0] {
            MalType::True
        } else {
            MalType::False
        })
    });
    ns.insert("symbol?", |args| {
        Ok(if let MalType::Symbol(_) = &args[0] {
            MalType::True
        } else {
            MalType::False
        })
    });
    ns.insert("symbol", |args| {
        if let MalType::String(value) = &args[0] {
            Ok(MalType::symbol(value))
        } else {
            Err(format!("Expected string, but got {}", &args[0]))
        }
    });
    ns.insert("keyword", |args| match &args[0] {
        MalType::String(value) => Ok(MalType::Keyword(value.to_string())),
        MalType::Keyword(value) => Ok(MalType::Keyword(value.clone())),
        _ => Err(format!("Expected string, but got {}", &args[0])),
    });
    ns.insert("keyword?", |args| {
        Ok(if let MalType::Keyword(_) = &args[0] {
            MalType::True
        } else {
            MalType::False
        })
    });
    ns.insert("vector", |args| Ok(MalType::Vector(args.clone())));
    ns.insert("vector?", |args| {
        Ok(if let MalType::Vector(_) = &args[0] {
            MalType::True
        } else {
            MalType::False
        })
    });
    ns.insert("sequential?", |args| {
        Ok(match &args[0] {
            MalType::List(_) | MalType::Vector(_) => MalType::True,
            _ => MalType::False,
        })
    });
    ns.insert("hash-map", |args| {
        let count = args.len();
        if count % 2 == 1 {
            return Err(format!(
                "Expected even number of args, but got {} of args",
                args.len()
            ));
        }

        let mut map = HashMap::new();
        for i in (0..count).step_by(2) {
            let key = match &args[i] {
                MalType::String(value) => Hashable::String(value.clone()),
                MalType::Keyword(value) => Hashable::Keyword(value.clone()),
                _ => return Err(format!("Expected string or keyword, but got {}", &args[i])),
            };
            let value = &args[i + 1];
            map.insert(key, value.clone());
        }
        Ok(MalType::Hashmap(map))
    });
    ns.insert("map?", |args| {
        Ok(if let MalType::Hashmap(_) = &args[0] {
            MalType::True
        } else {
            MalType::False
        })
    });
    ns.insert("assoc", |args| {
        let count = args.len();
        if count % 2 == 0 {
            return Err(format!(
                "Expected odd number of args, but got {} of args",
                args.len()
            ));
        }

        let mut map = match &args[0] {
            MalType::Hashmap(map) => map.clone(),
            _ => return Err(format!("Expected hashmap, but got {}", &args[0])),
        };
        for i in (1..count).step_by(2) {
            let key = match &args[i] {
                MalType::String(value) => Hashable::String(value.clone()),
                MalType::Keyword(value) => Hashable::Keyword(value.clone()),
                _ => return Err(format!("Expected string or keyword, but got {}", &args[i])),
            };
            let value = &args[i + 1];
            map.insert(key, value.clone());
        }
        Ok(MalType::Hashmap(map))
    });
    ns.insert("dissoc", |args| {
        let mut map = match &args[0] {
            MalType::Hashmap(map) => map.clone(),
            _ => return Err(format!("Expected hashmap, but got {}", &args[0])),
        };
        for key in &args[1..] {
            let key = match key {
                MalType::String(value) => Hashable::String(value.clone()),
                MalType::Keyword(value) => Hashable::Keyword(value.clone()),
                _ => return Err(format!("Expected string or keyword, but got {}", &key)),
            };
            map.remove(&key);
        }
        Ok(MalType::Hashmap(map))
    });
    ns.insert("get", |args| {
        let map = match &args[0] {
            MalType::Hashmap(map) => map.clone(),
            MalType::Nil => return Ok(MalType::Nil),
            _ => return Err(format!("Expected hashmap, but got {}", &args[0])),
        };
        let key = match &args[1] {
            MalType::String(value) => Hashable::String(value.clone()),
            MalType::Keyword(value) => Hashable::Keyword(value.clone()),
            _ => return Err(format!("Expected string or keyword, but got {}", &args[1])),
        };
        match map.get(&key) {
            Some(value) => Ok(value.clone()),
            None => Ok(MalType::Nil),
        }
    });
    ns.insert("contains?", |args| {
        let map = match &args[0] {
            MalType::Hashmap(map) => map.clone(),
            _ => return Err(format!("Expected hashmap, but got {}", &args[0])),
        };
        let key = match &args[1] {
            MalType::String(value) => Hashable::String(value.clone()),
            MalType::Keyword(value) => Hashable::Keyword(value.clone()),
            _ => return Err(format!("Expected string or keyword, but got {}", &args[1])),
        };
        match map.get(&key) {
            Some(_) => Ok(MalType::True),
            None => Ok(MalType::False),
        }
    });
    ns.insert("keys", |args| {
        let map = match &args[0] {
            MalType::Hashmap(map) => map.clone(),
            _ => return Err(format!("Expected hashmap, but got {}", &args[0])),
        };

        let mut keys = Vec::new();
        for key in map.keys() {
            keys.push(match key {
                Hashable::String(value) => MalType::String(value.clone()),
                Hashable::Keyword(value) => MalType::Keyword(value.clone()),
            });
        }

        Ok(MalType::List(keys))
    });
    ns.insert("vals", |args| {
        let map = match &args[0] {
            MalType::Hashmap(map) => map.clone(),
            _ => return Err(format!("Expected hashmap, but got {}", &args[0])),
        };

        Ok(MalType::List(
            map.values().map(|v| v.clone()).collect::<Vec<MalType>>(),
        ))
    });
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

fn read_file(filename: &str) -> Result<MalType, String> {
    fs::read_to_string(filename)
        .map(|v| MalType::String(v))
        .or_else(|err| Err(err.to_string()))
}
