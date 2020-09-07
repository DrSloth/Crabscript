use crate::{
    base::{DayFunction, DayObject},
    node::RootNode,
};
use std::{cell::UnsafeCell, collections::HashMap, sync::Arc};
use Var::*;

//The naming is a bit off... ugh
#[derive(Debug, Clone)]
pub enum Var {
    Const(DayObject),
    Variable(DayObject),
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
    //}Clone
}

//As soon as a multi threaded context is needed interior mutability and some unsafe is needed
#[derive(Default, Debug)]
pub struct Variables<'a> {
    vars: UnsafeCell<HashMap<String, Var>>,
    //NOTE the function definition seems kinda weird and is but i can't think of a better solution
    //maybe this can be done better, but anyway it's nasty if DayObject has lifetimes, maybe the
    //conflicts can be resolved... maybe
    funcs: UnsafeCell<Vec<RootNode<'a>>>,
}

fn undefined_variable(key: &str) -> ! {
    eprintln!("Access to undefined variable: {}", key);
    std::process::exit(1)
}

impl<'b, 'ret, 'a: 'ret> Variables<'a> {
    ///Creates an empty Variable Manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Receives a variable from the variable manager
    ///
    /// ### Panics
    /// Panics if the variable doesn't exist
    pub fn get_var(self: Arc<Self>, key: &'b str) -> DayObject {
        unsafe {
            if let Some(v) = (*self.vars.get()).get(key) {
                v.get()
            } else {
                undefined_variable(key)
            }
        }
    }

    ///Changes the value of a variable in the Variable Manager
    ///
    /// ### Panics
    /// Panics if the variable doesn't exist
    pub fn set_var(self: Arc<Self>, key: &str, value: DayObject) {
        unsafe {
            match (*self.vars.get()).get_mut(key) {
                None => undefined_variable(&key),
                Some(v) => match v {
                    Const(_) => panic!("Can't redefine constant {:?}", key),
                    Variable(v) => *v = value,
                },
            }
        }
    }

    ///Adds a variable to the Variable Manager
    ///
    /// ### Panics
    /// Panics if the variable already exist
    pub fn def_var(self: Arc<Self>, key: String, value: DayObject) {
        unsafe {
            let varmap = self.vars.get();
            match (*varmap).get(&key) {
                None => {
                    (*varmap).insert(key, Variable(value));
                }
                Some(_) => {
                    eprintln!("Redefinition of already defined variable: {}", key);
                    std::process::exit(1)
                }
            }
        }
    }

    /// Adds a constant to the Variable Manager
    ///
    /// ### Panics
    /// Panics if the variable already exist
    pub fn def_const(self: Arc<Self>, key: String, value: DayObject) {
        unsafe {
            let varmap = self.vars.get();
            match (*varmap).get(&key) {
                None => {
                    (*varmap).insert(key, Const(value));
                }
                Some(_) => {
                    eprintln!("Redefinition of already defined variable: {}", key);
                    std::process::exit(1)
                }
            }
        }
    }

    /// Adds a constant to the Variable Manager
    ///
    /// ### Panics
    /// Panics if the variable already exist
    pub fn populate_const(&self, key: String, value: DayObject) {
        unsafe {
            let varmap = self.vars.get();
            match (*varmap).get(&key) {
                None => {
                    (*varmap).insert(key, Const(value));
                }
                Some(_) => {
                    eprintln!("Redefinition of already defined variable: {}", key);
                    std::process::exit(1)
                }
            }
        }
    }

    /// Adds a constant to the Variable Manager
    ///
    /// ### Panics
    /// Panics if the variable already exist
    pub fn def_fn<'key>(self: Arc<Self>, key: String, value: RootNode<'a>) {
        unsafe {
            match (*self.vars.get()).get(&key) {
                None => {
                    self.clone().def_const(
                        key,
                        DayObject::Function(Arc::new(DayFunction::RuntimeDef(
                            (*self.funcs.get()).len(),
                        ))),
                    );
                    (*self.funcs.get()).push(value);
                }
                Some(v) => {
                    eprintln!("{} is already defined as {:?}", key, v);
                    std::process::exit(1)
                }
            }
        }
    }

    pub fn exec_fn(self: Arc<Self>, _args: crate::base::Args, key: usize) {
        unsafe {
            if let Some(v) = (*self.funcs.get()).get_mut(key) {
                //this is where we need scopes
                v.execute(Arc::clone(&self));
            } else {
                panic!("No function with id {}", key);
            }
        }
    }
}
