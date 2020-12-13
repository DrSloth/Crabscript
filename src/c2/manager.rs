use crate::base::{ArgVec, DayObject};
use std::{any::Any, cell::UnsafeCell, sync::Arc};

//TODO The var manager behavior should be extracted to an
//behavioral trait, this would be a big trait
//the complete API should be rethought for this and reduced a bit
//its ok to just add thoughts up to it right now

//NOTE for now i believe indexing by two usizes is easier, might be changed later

#[derive(Default)]
pub struct RuntimeManager {
    depth: usize,
    args: UnsafeCell<Option<Arc<UnsafeCell<ArgVec>>>>,
    ///The inner scope that is cleared when this manager is used
    inner_scope: UnsafeCell<Vec<Arc<UnsafeCell<DayObject>>>>,
    ///The outer scope of this scope that is not cleared on use
    //outer_scope: UnsafeCell<Vec<DayObject>>,
    ///The manager of the previous scope
    predecessor: Option<Arc<RuntimeManager>>,
    //I think this is only accessed by a single thread at a time
    //so it doesn't need synchronisation
    cache: UnsafeCell<Vec<Arc<dyn Cache>>>,
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

    //NOTE In theory that would an optimisation for runtime defs
    /*
    pub fn set_args(self: &Arc<Self>, args: &Vec<Node>) {
        if let Some(args) = self.args {

        } else {

        }
    } */

    pub fn get_predecessor(self: &Arc<Self>) -> Option<Arc<Self>> {
        self.predecessor.clone()
    }

    pub fn get_nth_predecessor<'a>(self: &'a Arc<Self>, depth: usize) -> &'a Arc<Self> {
        let mut current = self;
        for _ in 0..depth {
            //println!("!!!curd{}", current.depth);
            current = if let Some(c) = &current.predecessor {
                c
            } else {
                unreachable!("Access to non existant predecessor")
            }
        }

        current
    }

    /// Receives a variable from the variable manager
    ///
    /// ### Panics
    /// Panics if the variable doesn't exist
    pub fn get_var(self: &Arc<Self>, id: usize, depth: usize) -> DayObject {
        unsafe {
            let manager = if depth != self.depth {
                self.get_nth_predecessor(self.depth - depth)
            } else {
                self
            };
            let len = (*manager.inner_scope.get()).len();

            if id < len {
                (*(*manager.inner_scope.get())[id].get()).clone()
            } else {
                (*(*manager.inner_scope.get())[id - len].get()).clone()
            }
        }
    }

    pub fn get_var_cache(self: &Arc<Self>, id: usize, depth: usize) -> Arc<UnsafeCell<DayObject>> {
        unsafe {
            let manager = if depth != self.depth {
                self.get_nth_predecessor(self.depth - depth)
            } else {
                self
            };
            let len = (*manager.inner_scope.get()).len();

            if id < len {
                (*manager.inner_scope.get())[id].clone()
            } else {
                (*manager.inner_scope.get())[id - len].clone()
            }
        }
    }

    /// Retrieves a mutable reference to a variable
    ///
    /// ### Panics
    /// Panics if the variable doesn't exist
    pub fn get_var_mut<'a>(self: &Arc<Self>, id: usize, depth: usize) -> &'a mut DayObject {
        unsafe {
            let manager = if depth != self.depth {
                self.get_nth_predecessor(self.depth - depth)
            } else {
                self
            };
            let len = (*manager.inner_scope.get()).len();

            if id < len {
                &mut (*(*manager.inner_scope.get())[id].get())
            } else {
                &mut (*(*manager.inner_scope.get())[id - len].get())
            }
        }
    }

    ///Changes the value of a variable in the Variable Manager
    pub fn set_var(self: &Arc<Self>, value: DayObject, id: usize, depth: usize) {
        unsafe {
            let manager = if depth != self.depth {
                self.get_nth_predecessor(self.depth - depth)
            } else {
                self
            };

            let scptr = manager.inner_scope.get();
            let len = (*scptr).len();

            if id < len {
                *(*manager.inner_scope.get())[id].get() = value
            } else {
                *(*manager.inner_scope.get())[id - len].get() = value
            }
        }
    }

    ///Changes the value of a variable in the Variable Manager
    pub fn set_var_here(self: &Arc<Self>, value: DayObject, id: usize) {
        unsafe { *(*self.inner_scope.get()).get_unchecked(id).get() = value }
    }

    ///Adds a variable to the Variable Manager
    pub fn def_var(self: &Arc<Self>, id: usize, value: DayObject) {        
        unsafe {
            let inner = &mut *self.inner_scope.get();

            if inner.len() == id {
                inner.push(Arc::new(UnsafeCell::new(value)))
            } else if inner.len() > id {
                *inner[id].get() = value
            } else {
                unreachable!()
            }
         }
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

    pub fn get_args(self: &Arc<Self>) -> ArgVec {
        self.get_args_mut().clone()
    }

    pub fn get_args_mut<'a>(self: &Arc<Self>) -> &'a mut ArgVec {
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

    pub fn def_args(self: &Arc<Self>, args: Arc<UnsafeCell<ArgVec>>) {
        unsafe {
            *self.args.get() = Some(args);
        }
    }

    pub fn def_args_alloc(self: &Arc<Self>, args: ArgVec) {
        self.def_args(Arc::new(UnsafeCell::new(args)))
    }

    pub fn get_depth(self: &Arc<Self>) -> usize {
        self.depth
    }

    pub fn def_cache(self: &Arc<Self>, cache: Arc<dyn Cache>) -> CacheHandle {
        unsafe {
            let cptr = self.cache.get();
            (*cptr).push(cache);
            (*cptr).len() - 1
        }
    }

    pub fn get_cache(self: &Arc<Self>, handle: CacheHandle) -> Arc<dyn Cache> {
        unsafe {
            let cptr = self.cache.get();
            Arc::clone(&(*cptr)[handle])
        }
    }

    //clears the cache of this manager
    pub fn clear_cache(&self) {
        unsafe { (*self.cache.get()).clear() }
    }

    //clears the cache of this manager and all it's successors
    pub fn clear_all_cache(&self) {}
}

/// The Cache handle likely needs another identification field
/// for generational comparison and safety in multithreaded contexts
pub type CacheHandle = usize;

pub trait Cache: Any {
    fn get_cached(&self) -> DayObject;
    fn get_cached_mut<'a>(&self) -> &'a mut DayObject;
    fn do_cached(&self, value: DayObject);
    fn exec_cached(&self, value: DayObject) -> DayObject {
        self.do_cached(value);
        self.get_cached()
    }
}

pub struct IdentCache {
    cache: Arc<UnsafeCell<DayObject>>,
}

impl IdentCache {
    pub fn new(cache: Arc<UnsafeCell<DayObject>>) -> Self {
        Self { cache }
    }
}

impl Cache for IdentCache {
    fn get_cached(&self) -> DayObject {
        unsafe { (*self.cache.get()).clone() }
    }

    fn get_cached_mut<'a>(&self) -> &'a mut DayObject {
        unsafe { &mut (*self.cache.get()) }
    }

    fn do_cached(&self, value: DayObject) {
        unsafe { *self.cache.get() = value }
    }
}
