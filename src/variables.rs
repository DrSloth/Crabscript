use crate::base::DayObject;
use std::collections::HashMap;
use Var::*;

//The naming is a bit off... ugh
#[derive(Debug)]
pub enum Var {
    Const(DayObject),
    Variable(DayObject)
}

impl Var {
    pub fn get(&self) -> DayObject {
        match self {
            Const(v) => v.clone(),
            Variable(v) => v.clone(),
        }
    }

    //pub fn is_const(&self) -> bool {
    //    matches!(self, Const(_))
    //}
}

//As soon as a multi threaded context is needed interior mutability and some unsafe is needed
#[derive(Default, Debug)]
pub struct Variables {
    vars: HashMap<String, Var>,
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
            v.get()
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
            Some(v) => {
                match v {
                    Const(_) => panic!("Can't redefine constant {:?}", key),
                    Variable(v) => *v = value
                }
            },
        }
    }

    ///Adds a variable to the Variable Manager
    ///
    /// ###Panics
    /// Panics if the variable already exist
    pub fn def_var(&mut self, key: String, value: DayObject) {
        match self.vars.get(&key) {
            None => {
                self.vars.insert(key, Variable(value));
            }
            Some(_) => {
                eprintln!("Redefinition of already defined variable: {}", key);
                std::process::exit(1)
            }
        }
    }


    /// Adds a constant to the Variable Manager
    ///
    /// ###Panics
    /// Panics if the variable already exist
    pub fn def_const(&mut self, key: String, value: DayObject) {
        match self.vars.get(&key) {
            None => {
                self.vars.insert(key, Const(value));
            }
            Some(_) => {
                eprintln!("Redefinition of already defined constant: {}", key);
                std::process::exit(1)
            }
        }
    }
}
