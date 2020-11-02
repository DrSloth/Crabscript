use crate::base::{Args, DayObject};
use std::{cell::UnsafeCell, sync::Arc};

//TODO The var manager behavior should be extracted to an
//behavioral trait, this would be a big trait
//the complete API should be rethought for this and reduced a bit
//its ok to just add thoughts up to it right now

/* pub enum Function {
    RustFunc(Arc<dyn Fn(Args) -> DayObject>),
    Func(Arc<RootNode>, Arc<RuntimeManager>),
    Closure(Arc<RootNode>, Arc<RuntimeManager>),
} */

//NOTE for now i believe indexing by two usizes is easier, might be changed later

#[derive(Default)]
pub struct RuntimeManager {
    depth: usize,
    args: UnsafeCell<Option<Arc<UnsafeCell<Args>>>>,
    ///The inner scope that is cleared when this manager is used
    inner_scope: UnsafeCell<Vec<DayObject>>,
    ///The outer scope of this scope that is not cleared on use
    outer_scope: UnsafeCell<Vec<DayObject>>,
    ///The manager of the previous scope
    predecessor: Option<Arc<RuntimeManager>>,
}

unsafe impl Send for RuntimeManager {}
unsafe impl Sync for RuntimeManager {}

impl RuntimeManager {
    ///Creates an empty Variable Manager
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_capacity(inner_capacity: usize) -> Self {
        Self {
            inner_scope: UnsafeCell::new(Vec::with_capacity(inner_capacity)),
            ..Self::default()
        }
    }

    pub fn new_capacity_predecessor(inner_capacity: usize, predecessor: Option<Arc<Self>>) -> Self {
        Self {
            inner_scope: UnsafeCell::new(Vec::with_capacity(inner_capacity)),
            depth: predecessor.as_ref().map(|p| p.depth + 1).unwrap_or(0),
            predecessor,
            ..Default::default()
        }
    }

    pub fn get_predecessor(self: &Arc<Self>) -> Option<Arc<Self>> {
        self.predecessor.clone()
    }

    pub fn get_nth_predecessor(self: &Arc<Self>, depth: usize) -> Arc<Self> {
        let mut current = self;
        for _ in 0..depth {
            //println!("!!!curd{}", current.depth);
            current = if let Some(c) = &current.predecessor {
                c
            } else {
                unreachable!("Access to non existant predecessor")
            }
        }

        //println!("!!!mgrd{}", current.depth);
        Arc::clone(current)
    }

    /// Receives a variable from the variable manager
    ///
    /// ### Panics
    /// Panics if the variable doesn't exist
    pub fn get_var(self: &Arc<Self>, id: usize, depth: usize) -> DayObject {
        unsafe {
            let manager = self.get_nth_predecessor(self.depth - depth);
            let len = (*manager.inner_scope.get()).len();

            if id < len {
                (*manager.inner_scope.get())[id].clone()
            } else {
                (*manager.inner_scope.get())[id - len].clone()
            }
        }
    }

    /// Retrieves a mutable pointer to a variable
    ///
    /// ### Panics
    /// Panics if the variable doesn't exist
    pub fn get_var_mut<'a>(self: &Arc<Self>, id: usize, depth: usize) -> &'a mut DayObject {
        unsafe {
            let manager = self.get_nth_predecessor(self.depth - depth);
            let len = (*manager.inner_scope.get()).len();

            if id < len {
                (*manager.inner_scope.get()).get_mut(id).unwrap()
            } else {
                (*manager.inner_scope.get()).get_mut(id - len).unwrap()
            }
        }
    }

    ///Changes the value of a variable in the Variable Manager
    ///
    /// ### Panics
    /// Panics if the variable doesn't exist
    pub fn set_var(self: &Arc<Self>, value: DayObject, id: usize, depth: usize) {
        unsafe {
            let manager = self.get_nth_predecessor(self.depth - depth);

            let len = (*manager.inner_scope.get()).len();

            if id < len {
                (*manager.inner_scope.get())[id] = value
            } else {
                (*manager.inner_scope.get())[id - len] = value
            }
        }
    }

    ///Adds a variable to the Variable Manager
    ///
    /// ### Panics
    /// Panics if the variable already exist
    pub fn def_var(self: &Arc<Self>, value: DayObject) {
        unsafe { (*self.inner_scope.get()).push(value) }
    }

    pub fn clear(self: &Arc<Self>) {
        unsafe { (*self.inner_scope.get()).clear() }
    }

    pub fn get_new_scope(self: &Arc<Self>) -> Arc<Self> {
        Arc::new(Self {
            predecessor: Some(Arc::clone(self)),
            ..Default::default()
        })
    }

    //TODO Should args be mutable?

    pub fn get_args(self: &Arc<Self>) -> Args {
        self.get_args_mut().clone()
    }

    pub fn get_args_mut<'a>(self: &Arc<Self>) -> &'a mut Args {
        let mut current = self;
        loop {
            unsafe {
                if let Some(args) = &*current.args.get() {
                    *self.args.get() = Some(Arc::clone(args));
                    return &mut *args.get();
                }
            }
            current = if let Some(pre) = &current.predecessor {
                pre
            } else {
                panic!("No args found")
            }
        }
    }

    pub fn get_arg(self: &Arc<Self>, id: usize) -> DayObject {
        self.get_args_mut()[id].clone()
    }

    pub fn def_args(self: &Arc<Self>, args: Arc<UnsafeCell<Args>>) {
        unsafe {
            *self.args.get() = Some(args);
        }
    }

    pub fn def_args_alloc(self: &Arc<Self>, args: Args) {
        self.def_args(Arc::new(UnsafeCell::new(args)))
    }

    /*
    /// Adds a function to the Variable Manager
    ///
    /// ### Panics
    /// Panics if the function already exists in the current scope
    pub fn def_fn<'key>(self: &Arc<Self>, hash: u64, key: &'a str, value: Arc<RootNode>) -> usize {
        unsafe {
            match (*self.inner_scope.get()).get(&key) {
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

    pub fn def_closure(self: &Arc<Self>, value: Arc<RootNode>) -> usize {
        unsafe {
            let len = (*self.funcs.get()).len();
            (*self.funcs.get()).push(Function::Closure(value, self.get_new_scope()));
            len
        }
    }

    pub fn def_rust_func(self: &Arc<Self>, value: Arc<dyn Fn(Args) -> DayObject>) -> usize {
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
    */
}

/* use std::sync::RwLock;

pub struct CrabJoinHandle(RwLock<CrabJoinHandleInner>);

impl CrabJoinHandle {
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

enum CrabJoinHandleInner {
    Pending(Option<JoinGuard<'a, DayObject>>),
    Value(DayObject),
}

impl Debug for CrabJoinHandle {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        use CrabJoinHandleInner::*;
        match &*self.0.read().expect("Error Reading") {
            Pending(_) => write!(f, "Pending"),
            Value(val) => write!(f, "{:?}", val),
        }
    }
}
 */
