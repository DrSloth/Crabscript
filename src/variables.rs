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
    funcs: Arc<UnsafeCell<Vec<RootNode<'a>>>>,
    predecessor: Option<Arc<Variables<'a>>>,
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

    pub fn new_scope(self: Arc<Self>) -> Arc<Self> {
        Arc::new(Self {
            predecessor: Some(Arc::clone(&self)),
            funcs: Arc::clone(&self.funcs),
            ..Default::default()
        })
    }

    pub fn pop_scope(self) -> Option<Arc<Self>> {
        self.predecessor
    }

    /// Receives a variable from the variable manager
    ///
    /// ### Panics
    /// Panics if the variable doesn't exist
    pub fn get_var(self: Arc<Self>, key: &'b str) -> DayObject {
        unsafe {
            let mut current = self;
            loop {
                if let Some(v) = (*current.vars.get()).get(key) {
                    return v.get();
                } else {
                    if let Some(next) = &current.predecessor {
                        current = Arc::clone(next);
                    } else {
                        undefined_variable(key)
                    }
                }
            }
        }
    }

    ///Changes the value of a variable in the Variable Manager
    ///
    /// ### Panics
    /// Panics if the variable doesn't exist
    pub fn set_var(self: Arc<Self>, key: &str, value: DayObject) {
        let mut current = self;
        unsafe {
            loop {
                match (*current.vars.get()).get_mut(key) {
                    None => {
                        if let Some(next) = &current.predecessor {
                            current = Arc::clone(next);
                        } else {
                            undefined_variable(key)
                        }
                    }
                    Some(v) => match v {
                        Const(_) => panic!("Can't redefine constant {:?}", key),
                        Variable(v) => {
                            *v = value;
                            break;
                        }
                    },
                }
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

    pub fn exec_fn(self: Arc<Self>, args: crate::base::Args, key: usize) {
        unsafe {
            if let Some(v) = (*self.funcs.get()).get_mut(key) {
                let scope = self.new_scope();

                Arc::clone(&scope).def_const("args".to_string(), DayObject::Array(args));

                v.execute(Arc::clone(&scope));
            } else {
                panic!("No function with id {}", key);
            }
        }
    }
}

//TODO For debugging purposes a Drop on DayObject and this Variable manager should be done this would be feature gated
//behind the debug flag, it shouldn't be hard to implement aspacially with an inner in the impl
