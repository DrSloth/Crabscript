use generational_arena::{Arena, Index};
//use super::global_manager;

type MgrIdx = Index;

#[derive(Debug)]
pub struct ManagedArena<T> {
    inner: Arena<T>,
}

impl<T> ManagedArena<T> {
    pub fn new() -> Self {
        Self {
            inner: Arena::new(),
        }
    }

    pub fn insert(&mut self, item: T) {
        self.inner.insert(item);
    }
}

/// A handle to data in a ManagedArena
/// When this handle is dropped it notifies the manager
/// holding the arena this is from
#[derive(Clone)]
pub struct ArenaHandle {
    idx: Index,
    manager_idx: MgrIdx,
}

impl ArenaHandle {
    pub fn new(idx: Index, manager_idx: MgrIdx) -> Self {
        Self { idx, manager_idx }
    }
}

/*impl Drop for ArenaHandle {
    fn drop(&mut self) {
        global_manager::notify_drop(self)
    }
}*/
