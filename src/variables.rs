use std::collections::HashMap;
use pgc::Gc;
use crate::base::{BaseType, DayObject};

//As soon as a multi threaded context is needed interior mutability is needed 
#[derive(Default)]
pub struct Variables {
    vars: HashMap<String, BaseType>,
}

fn undefined_variable(key: &str) -> ! {
    eprintln!("Access to undefined variable: {}", key);
    std::process::exit(1)
}

impl Variables {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_var(&self, key: &str) -> BaseType {
        if let Some(v) = self.vars.get(key) {
            *v
        } else {
            undefined_variable(key)
        }
    }

    pub fn set_var(&mut self, key: &str, value: DayObject) {
        match self.vars.get_mut(key) {
            None => undefined_variable(&key),
            Some(gc) => {
                *gc.get_mut() = value
            }
        } 
    }

    pub fn def_var(&mut self, key: String, value: DayObject) {
        match self.vars.get(&key) {
            None => {
                let gc = Gc::new(value);
                pgc::add_root(gc);
                self.vars.insert(key, gc);
            },
            Some(_) => {
                eprintln!("Redefinition of already defined variable: {}", key);
                std::process::exit(1)
            }
        }
    }
}