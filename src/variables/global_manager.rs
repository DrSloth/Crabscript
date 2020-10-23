use super::{managed_arena::{ArenaHandle, ManagedArena}, variables::Variables};
use lazy_static::lazy_static;
use std::sync::{RwLock, Weak};

//NOTE The current approach uses Pins and pointers to access all defined var managers.
//the efficiency implications of that have to be explored
//this should also be non public so that instructions still have get a var manager
//passed in order to access it

lazy_static! {
    static ref MANAGER_MAP: RwLock<ManagedArena<Weak<Variables<'static>>>> = {
        RwLock::new(ManagedArena::new())
    };
}

pub fn notify_drop(handle: &ArenaHandle) {
    todo!()
}

