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
    ///Creates an empty Variable Manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Receives a variable from the variable manager
    /// 
    /// ###Panics
    /// Panics if the variable doesn't exist
    pub fn get_var(&self, key: &str) -> DayObject {
        if let Some(v) = self.vars.get(key) {
            v.clone()
        } else {
            undefined_variable(key)
        }
    }
    
    ///Changes the value of a variable in the Variable Manager
    /// 
    /// ###Panics
    /// Panics if the variable doesn't exist
    pub fn set_var(&mut self, key: &str, value: DayObject) {
        match self.vars.get_mut(key) {
            None => undefined_variable(&key),
            Some(r) => {
                *r = value
            }
        } 
    }
    
    ///Adds a variable to the Variable Manager
    ///
    /// ###Panics
    /// Panics if the variable already exist
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