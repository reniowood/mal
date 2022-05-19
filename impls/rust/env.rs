use std::{collections::HashMap, rc::Rc, cell::RefCell};

use crate::types::MalType;

pub struct Env {
    outer: Option<Rc<RefCell<Env>>>,
    data: HashMap<String, MalType>,
}

impl Env {
    pub fn new(outer: Option<Rc<RefCell<Env>>>) -> Self {
        Env { outer, data: HashMap::new() }
    }

    pub fn set(&mut self, key: String, value: MalType) {
        self.data.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<MalType> {
        if let Some(value) = self.data.get(key) {
            return Some(value.clone());
        }

        self.outer.as_ref().map(|outer| outer.borrow().get(key)).flatten()
    }
}