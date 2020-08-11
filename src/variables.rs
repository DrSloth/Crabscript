use std::collections::HashMap;
use crate::base::{DayObject};

//As soon as a multi threaded context is needed interior mutability and some unsafe is needed 
#[derive(Default, Debug)]
pub struct Variables {
    vars: HashMap<String, DayObject>,
}

fn undefined_variable(key: &str) -> ! {
    eprintln!("Access to undefined variable: {}", key);
    std::process::exit(1)
}

impl Variables {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_var(&self, key: &str) -> DayObject {
        if let Some(v) = self.vars.get(key) {
            v.clone()
        } else {
            undefined_variable(key)
        }
    }

    pub fn set_var(&mut self, key: &str, value: DayObject) {
        match self.vars.get_mut(key) {
            None => undefined_variable(&key),
            Some(r) => {
                *r = value
            }
        } 
    }

    pub fn def_var(&mut self, key: String, value: DayObject) {
        match self.vars.get(&key) {
            None => {
                self.vars.insert(key, value);
            },
            Some(_) => {
                eprintln!("Redefinition of already defined variable: {}", key);
                std::process::exit(1)
            }
        }
    }
}