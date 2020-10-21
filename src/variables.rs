use crate::{
    base::{Args, DayFunction, DayObject},
    node::RootNode,
};
use ahash::RandomState as AHasherBuilder;
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

    pub fn get_mut(&mut self) -> &mut DayObject {
        match self {
            Const(v) => v,
            Variable(v) => v,
        }
    }
}

//#[derive()]
pub enum Function<'a> {
    RustFunc(Arc<dyn Fn(Args) -> DayObject + 'a>),
    Func(Arc<RootNode<'a>>, Arc<Variables<'a>>),
    Closure(Arc<RootNode<'a>>, Arc<Variables<'a>>),
}

//As soon as a multi threaded context is needed interior mutability and some unsafe is needed
#[derive(Default)]
pub struct Variables<'a> {
    predecessor: Option<Arc<Variables<'a>>>,
    vars: UnsafeCell<HashMap<String, Var, AHasherBuilder>>,
    //NOTE Maybe this needs a better system where func indices can be reused
    //maybe this should be feature gated with one feature gate to make it runtime
    //decidable through a Variables trait that should be default for the
    //interpreter binary. In niche cases the Vec might run out of space or take too much
    //space, the reinsertion system would need some work. (In generall this language should value
    //runtime customisability just as compile time customisability to get the last drop of da cpus
    funcs: Arc<UnsafeCell<Vec<Function<'a>>>>,
    //iter_arena: Arc<UnsafeCell<Vec<Option<Arc<dyn IterData<'a>>>>>>,
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
            //iter_arena: Arc::clone(&self.iter_arena),
            ..Default::default()
        })
    }

    pub fn get_new_scope(self: &Arc<Self>) -> Arc<Self> {
        Arc::new(Self {
            predecessor: Some(Arc::clone(self)),
            funcs: Arc::clone(&self.funcs),
            //iter_arena: Arc::clone(&self.iter_arena),
            ..Default::default()
        })
    }

    pub fn pop_scope(self) -> Option<Arc<Self>> {
        //TODO Either Drop has to be implemented for
        //var manager or scopes have to be deleted manually
        //in order to implement the fn drop system
        println!("Scope popped");
        self.predecessor
    }

    pub fn previous_scope(self: Arc<Self>) -> Option<Arc<Self>> {
        if let Some(p) = &self.predecessor {
            Some(Arc::clone(p))
        } else {
            None
        }
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

    /// Retrieves a mutable pointer to a variable
    ///
    /// ### Panics
    /// Panics if the variable doesn't exist
    pub fn get_var_mut(self: Arc<Self>, key: &'b str) -> &mut DayObject {
        unsafe {
            let mut current = self;
            loop {
                if let Some(v) = (*current.vars.get()).get_mut(key) {
                    return v.get_mut();
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

    /// Adds a function to the Variable Manager
    ///
    /// ### Panics
    /// Panics if the function already exists in the current scope
    pub fn def_fn<'key>(self: Arc<Self>, key: String, value: Arc<RootNode<'a>>) -> usize {
        unsafe {
            match (*self.vars.get()).get(&key) {
                None => {
                    let len = (*self.funcs.get()).len();
                    self.clone()
                        .def_const(key, DayObject::Function(DayFunction::RuntimeDef(len)));
                    (*self.funcs.get()).push(Function::Func(value, Arc::clone(&self)));
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

    pub fn def_rust_func(self: &Arc<Self>, value: Arc<dyn Fn(Args) -> DayObject + 'a>) -> usize {
        unsafe {
            let len = (*self.funcs.get()).len();
            (*self.funcs.get()).push(Function::RustFunc(value));
            len
        }
    }

    pub fn exec_fn(self: Arc<Self>, args: Args, key: usize) -> DayObject {
        unsafe {
            if let Some(v) = (*self.funcs.get()).get_mut(key) {
                match v {
                    Function::Func(v, scope) => {
                        let scope = Arc::clone(scope).new_scope();
                        Arc::clone(&scope).def_const("args".to_string(), DayObject::Array(args));
                        v.execute(Arc::clone(&scope)).value()
                    }
                    Function::Closure(v, scope) => {
                        let scope = Arc::clone(scope).new_scope();
                        Arc::clone(&scope).def_const("args".to_string(), DayObject::Array(args));
                        v.execute(Arc::clone(&scope)).value()
                    }
                    Function::RustFunc(f) => f(args),
                }
            } else {
                panic!("No function with id {}", key);
            }
        }
    }
}

//TODO This should be somehow possible with an inner field being the struct
//it would also be possible to only run cleanup on inserting a new fn

/*impl Drop for Variables<'_> {
    fn drop(&mut self) {
        println!("Var mgr dropped")
    }
}*/

//TODO For debugging purposes a Drop on DayObject and this Variable manager should be done this would be feature gated
//behind the debug flag, it shouldn't be hard to implement with an inner in the impl
