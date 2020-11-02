use crate::{
    base::{Args, DayFunction, DayObject, ThreadId},
    node::RootNode,
};
use ahash::{AHasher, RandomState};
use hashbrown::{hash_map::RawEntryMut, HashMap};
use lazy_static::lazy_static;
use std::{
    cell::UnsafeCell,
    fmt::{Debug, Formatter, Result as FmtResult},
    sync::Arc,
};
use thread_scoped::*;
use Var::*;

lazy_static! {
    static ref ARGS_HASH: u64 = super::hash("args");
}
use std::hash::{BuildHasher, Hash, Hasher};

pub type GlobalHasherBuilder = RandomState;
pub type GlobalHasher = AHasher;

lazy_static! {
    static ref GLOBAL_HASHER_BUILDER: GlobalHasherBuilder = RandomState::with_seeds(1, 9, 8, 4);
    static ref GLOBAL_HASHER: GlobalHasher = GLOBAL_HASHER_BUILDER.build_hasher();
}

pub fn get_global_hasherbuilder() -> GlobalHasherBuilder {
    GLOBAL_HASHER_BUILDER.clone()
}

pub fn get_global_hasher() -> GlobalHasher {
    (&*GLOBAL_HASHER).clone()
}

pub fn hash(ident: &str) -> u64 {
    let mut hasher = get_global_hasher();
    ident.hash(&mut hasher);
    hasher.finish()
}

pub type ExecutionManager<'a> = Variables<'a>;

//TODO The var manager behavior should be extracted to an
//behavioral trait, this would be a big trait
//the complete API should be rethought for this and reduced a bit
//its ok to just add thoughts up to it right now

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

pub enum Function<'a> {
    RustFunc(Arc<dyn Fn(Args) -> DayObject + 'a>),
    Func(Arc<RootNode<'a>>, Arc<Variables<'a>>),
    Closure(Arc<RootNode<'a>>, Arc<Variables<'a>>),
}

#[derive(Default)]
pub struct Variables<'a> {
    predecessor: Option<Arc<Variables<'a>>>,
    vars: UnsafeCell<HashMap<&'a str, Var, RandomState>>,
    //TODO Reinsertion system
    funcs: Arc<UnsafeCell<Vec<Function<'a>>>>,
    threads: Arc<UnsafeCell<Vec<CrabJoinHandle<'a>>>>,
    //TODO Lifetime Managed memory Arena, also needs a reinsertion/GC System
}

unsafe impl Send for Variables<'_> {}
unsafe impl Sync for Variables<'_> {}

fn undefined_variable(key: &str) -> ! {
    panic!("Access to undefined variable: {}", key)
}

impl<'b, 'ret, 'a: 'ret> Variables<'a> {
    ///Creates an empty Variable Manager
    pub fn new() -> Self {
        Self {
            vars: UnsafeCell::new(HashMap::with_hasher(get_global_hasherbuilder())),
            ..Default::default()
        }
    }

    pub fn clear_scope_ref(self: &Arc<Self>) {
        unsafe { (*self.vars.get()).clear() }
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
    pub fn get_var_mut(self: Arc<Self>, key: &'a str) -> &mut DayObject {
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

    pub fn get_var_hash(self: &Arc<Self>, hash: u64, key: &'b str) -> DayObject {
        unsafe {
            let mut current = self;
            loop {
                if let Some((_k, v)) = (*current.vars.get())
                    .raw_entry()
                    .from_key_hashed_nocheck(hash, key)
                {
                    return v.get();
                } else {
                    if let Some(next) = &current.predecessor {
                        current = next;
                    } else {
                        undefined_variable(key)
                    }
                }
            }
        }
    }

    pub fn get_var_mut_hash(self: &Arc<Self>, hash: u64, key: &'a str) -> &mut DayObject {
        unsafe {
            let mut current = self;
            loop {
                match (*current.vars.get())
                    .raw_entry_mut()
                    .from_key_hashed_nocheck(hash, key)
                {
                    RawEntryMut::Occupied(e) => return e.into_mut().get_mut(),
                    RawEntryMut::Vacant(_) => {
                        if let Some(next) = &current.predecessor {
                            current = next;
                        } else {
                            undefined_variable(key)
                        }
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
        self.set_var_ref(key, value)
    }

    pub fn set_var_ref(self: &Arc<Self>, key: &str, value: DayObject) {
        let mut current = self;
        unsafe {
            loop {
                match (*current.vars.get()).get_mut(key) {
                    None => {
                        if let Some(next) = &current.predecessor {
                            current = next;
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

    pub fn set_var_ref_hash(self: &Arc<Self>, hash: u64, key: &str, value: DayObject) {
        unsafe {
            let mut current = self;
            loop {
                match (*current.vars.get())
                    .raw_entry_mut()
                    .from_key_hashed_nocheck(hash, key)
                {
                    RawEntryMut::Occupied(mut e) => match e.get_mut() {
                        Const(_) => panic!("Not allowed to redefine constant"),
                        Variable(v) => {
                            *v = value;
                            break;
                        }
                    },
                    RawEntryMut::Vacant(_) => {
                        if let Some(next) = &current.predecessor {
                            current = next;
                        } else {
                            undefined_variable(key)
                        }
                    }
                }
            }
        }
    }

    pub fn def_var(self: Arc<Self>, key: &'a str, value: DayObject) {
        self.def_var_ref(key, value)
    }

    ///Adds a variable to the Variable Manager
    ///
    /// ### Panics
    /// Panics if the variable already exist
    pub fn def_var_ref(self: &Arc<Self>, key: &'a str, value: DayObject) {
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

    pub fn def_var_ref_hash(self: &Arc<Self>, hash: u64, key: &'a str, value: DayObject) {
        unsafe {
            match (*self.vars.get())
                .raw_entry_mut()
                .from_key_hashed_nocheck(hash, key)
            {
                RawEntryMut::Occupied(_) => {
                    panic!("Redefinition of already defined variable: {}", key)
                }
                RawEntryMut::Vacant(e) => {
                    e.insert_hashed_nocheck(hash, key, Variable(value));
                }
            }
        }
    }

    pub fn def_const(self: Arc<Self>, key: &'a str, value: DayObject) {
        self.def_const_ref(key, value)
    }

    /// Adds a constant to the Variable Manager
    ///
    /// ### Panics
    /// Panics if the variable already exist
    pub fn def_const_ref(self: &Arc<Self>, key: &'a str, value: DayObject) {
        unsafe {
            let varmap = self.vars.get();
            match (*varmap).get(key) {
                None => {
                    (*varmap).insert(key, Const(value));
                }
                Some(_) => panic!("Redefinition of already defined variable: {}", key),
            }
        }
    }

    pub fn def_const_ref_hash(self: &Arc<Self>, hash: u64, key: &'a str, value: DayObject) {
        unsafe {
            match (*self.vars.get())
                .raw_entry_mut()
                .from_key_hashed_nocheck(hash, key)
            {
                RawEntryMut::Occupied(_) => {
                    panic!("Redefinition of already defined Const: {}", key)
                }
                RawEntryMut::Vacant(e) => {
                    e.insert_hashed_nocheck(hash, key, Const(value));
                }
            }
        }
    }

    /// Adds a constant to the Variable Manager
    ///
    /// ### Panics
    /// Panics if the variable already exist
    pub fn populate_const(&self, key: &'a str, value: DayObject) {
        unsafe {
            let varmap = self.vars.get();
            match (*varmap).get(key) {
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
    pub fn def_fn<'key>(
        self: &Arc<Self>,
        hash: u64,
        key: &'a str,
        value: Arc<RootNode<'a>>,
    ) -> usize {
        unsafe {
            match (*self.vars.get()).get(&key) {
                None => {
                    let len = (*self.funcs.get()).len();
                    self.clone().def_const_ref_hash(
                        hash,
                        key,
                        DayObject::Function(DayFunction::RuntimeDef(len)),
                    );
                    (*self.funcs.get()).push(Function::Func(value, self.get_new_scope()));
                    len
                }
                Some(v) => panic!("{} is already defined as {:?}", key, v),
            }
        }
    }

    pub fn def_closure(self: &Arc<Self>, value: Arc<RootNode<'a>>) -> usize {
        unsafe {
            let len = (*self.funcs.get()).len();
            (*self.funcs.get()).push(Function::Closure(value, self.get_new_scope()));
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
        self.exec_fn_ref(args, key)
    }

    pub fn exec_fn_ref(self: &Arc<Self>, args: Args, key: usize) -> DayObject {
        unsafe {
            if let Some(v) = (*self.funcs.get()).get(key) {
                match v {
                    Function::Func(v, outer_scope) => {
                        let scope = outer_scope.get_new_scope();
                        scope.def_const_ref_hash(*ARGS_HASH, "args", DayObject::Array(args));
                        v.execute(&scope).value()
                    }
                    Function::Closure(v, outer_scope) => {
                        let scope = outer_scope.get_new_scope();
                        scope.def_const_ref_hash(*ARGS_HASH, "args", DayObject::Array(args));
                        v.execute(&scope).value()
                    }
                    Function::RustFunc(f) => f(args),
                }
            } else {
                panic!("No function with id {}", key);
            }
        }
    }

    pub fn spawn_thread(self: Arc<Self>, exec: DayObject, mut args: Args) -> ThreadId {
        unsafe {
            let len = (*self.threads.get()).len();
            (*self.threads.get()).push(CrabJoinHandle::pending(thread_scoped::scoped(move || {
                match exec {
                    DayObject::Function(f) => f.call(args, &self.get_new_scope()),
                    DayObject::Iter(iter) => {
                        crate::std_modules::iter::collect_inner(iter, &self.get_new_scope())
                    }
                    val => {
                        args.insert(0, val);
                        DayObject::Array(args)
                    }
                }
            })));
            len
        }
    }

    pub fn join_thread(self: &Arc<Self>, id: ThreadId) -> DayObject {
        unsafe {
            let thptr = self.threads.get();
            (*thptr)[id].join()
        }
    }
}

use std::sync::RwLock;

pub struct CrabJoinHandle<'a>(RwLock<CrabJoinHandleInner<'a>>);

impl<'a> CrabJoinHandle<'a> {
    fn join(&self) -> DayObject {
        let lock = self.0.read().expect("Error reading");
        if let CrabJoinHandleInner::Value(val) = &*lock {
            return val.clone();
        }

        std::mem::drop(lock);
        let mut lock = self.0.write().expect("Error writing");
        let val = match &mut *lock {
            CrabJoinHandleInner::Pending(guard) => guard.take().unwrap().join(),
            CrabJoinHandleInner::Value(v) => v.clone(),
        };

        *lock = CrabJoinHandleInner::Value(val.clone());
        val
    }

    fn pending(guard: JoinGuard<'a, DayObject>) -> Self {
        Self(RwLock::new(CrabJoinHandleInner::Pending(Some(guard))))
    }
}

enum CrabJoinHandleInner<'a> {
    Pending(Option<JoinGuard<'a, DayObject>>),
    Value(DayObject),
}

impl Debug for CrabJoinHandle<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        use CrabJoinHandleInner::*;
        match &*self.0.read().expect("Error Reading") {
            Pending(_) => write!(f, "Pending"),
            Value(val) => write!(f, "{:?}", val),
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
