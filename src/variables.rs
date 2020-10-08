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
}

#[derive(Debug)]
pub enum Function<'a> {
    Func(Arc<RootNode<'a>>),
    Closure(Arc<RootNode<'a>>, Arc<Variables<'a>>),
}

//As soon as a multi threaded context is needed interior mutability and some unsafe is needed
#[derive(Default, Debug)]
pub struct Variables<'a> {
    vars: UnsafeCell<HashMap<String, Var>>,
    //NOTE the function definition seems kinda weird and is but i can't think of a better solution
    //maybe this can be done better, but anyway it's nasty if DayObject has lifetimes, maybe the
    //conflicts can be resolved... maybe
    funcs: Arc<UnsafeCell<Vec<Function<'a>>>>,
    predecessor: Option<Arc<Variables<'a>>>,
}

fn undefined_variable(key: &str) -> ! {
    panic!("Access to undefined variable: {}", key)
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
                Some(_) => panic!("Redefinition of already defined variable: {}", key),
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
                Some(_) => panic!("Redefinition of already defined variable: {}", key),
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
                    panic!("Redefinition of already defined variable: {}", key);
                }
            }
        }
    }

    /// Adds a constant to the Variable Manager
    ///
    /// ### Panics
    /// Panics if the variable already exist
    pub fn def_fn<'key>(self: Arc<Self>, key: String, value: Arc<RootNode<'a>>) -> usize {
        unsafe {
            match (*self.vars.get()).get(&key) {
                None => {
                    let len = (*self.funcs.get()).len();
                    self.clone()
                        .def_const(key, DayObject::Function(DayFunction::RuntimeDef(len)));
                    (*self.funcs.get()).push(Function::Func(value));
                    len
                }
                Some(v) => panic!("{} is already defined as {:?}", key, v),
            }
        }
    }

    pub fn populate_fn<'key>(&self, key: String, value: Arc<RootNode<'a>>) -> usize {
        unsafe {
            match (*self.vars.get()).get(&key) {
                None => {
                    let len = (*self.funcs.get()).len();
                    self.clone()
                        .populate_const(key, DayObject::Function(DayFunction::RuntimeDef(len)));
                    (*self.funcs.get()).push(Function::Func(value));
                    len
                }
                Some(v) => panic!("{} is already defined as {:?}", key, v),
            }
        }
    }

    pub fn def_closure(self: &Arc<Self>, value: Arc<RootNode<'a>>) -> usize {
        unsafe {
            let len = (*self.funcs.get()).len();
            (*self.funcs.get()).push(Function::Closure(value, Arc::clone(self)));
            len
        }
    }

    pub fn exec_fn(self: Arc<Self>, args: crate::base::Args, key: usize) -> DayObject {
        unsafe {
            if let Some(v) = (*self.funcs.get()).get_mut(key) {
                match v {
                    Function::Func(v) => {
                        let scope = self.new_scope();

                        Arc::clone(&scope).def_const("args".to_string(), DayObject::Array(args));

                        v.execute(Arc::clone(&scope)).value()
                    }
                    Function::Closure(v, scope) => {
                        let scope = Arc::clone(scope).new_scope();
                        Arc::clone(&scope).def_const("args".to_string(), DayObject::Array(args));
                        v.execute(Arc::clone(&scope)).value()
                    }
                }
            } else {
                panic!("No function with id {}", key);
            }
        }
    }
}

//TODO For debugging purposes a Drop on DayObject and this Variable manager should be done this would be feature gated
//behind the debug flag, it shouldn't be hard to implement with an inner in the impl
