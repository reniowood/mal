use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::types::MalType;

pub struct Env {
    outer: Option<Rc<RefCell<Env>>>,
    data: HashMap<String, MalType>,
}

impl Env {
    pub fn new(outer: Option<Rc<RefCell<Env>>>) -> Self {
        Env {
            outer,
            data: HashMap::new(),
        }
    }

    pub fn from(
        outer: Option<Rc<RefCell<Env>>>,
        binds: &Vec<MalType>,
        exprs: &Vec<MalType>,
    ) -> Self {
        let mut data = HashMap::new();
        for i in 0..binds.len() {
            let bind = &binds[i];
            let expr = if i < exprs.len() {
                &exprs[i]
            } else {
                &MalType::Nil
            };

            if let MalType::Symbol(name) = bind {
                if name == "&" {
                    let next = &binds[i + 1];
                    if let MalType::Symbol(name) = next {
                        data.insert(name.clone(), MalType::List(exprs[i..].to_vec()));
                    }

                    break;
                }

                data.insert(name.clone(), expr.clone());
            }
        }
        Env { outer, data }
    }

    pub fn set(&mut self, key: String, value: MalType) {
        self.data.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<MalType> {
        if let Some(value) = self.data.get(key) {
            return Some(value.clone());
        }

        self.outer
            .as_ref()
            .map(|outer| outer.borrow().get(key))
            .flatten()
    }
}
