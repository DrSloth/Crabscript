use crate::base::DayObject;
use std::{sync::Arc};

pub mod range;
//mod arr_iter;

/// This is used inside the var manager as backing data for iterators
pub trait IterData {
    fn acquire(self: Arc<Self>) -> Box<dyn Iter>;
    fn consume(self) -> Box<dyn Iter>;
    fn get_indexed(&self, _index: usize) -> Option<DayObject> {
        None
    }
}

/// A CrabScript iterator
pub trait Iter {
    /// Get the next element of the iter
    fn next(&mut self) -> Option<DayObject>;
    fn get_indexed(&self, index: usize) -> Option<DayObject>;
    /// Get which kind of iter this is
    fn kind(&self) -> IterKind;
    /// Returns a rewound version of this iterator
    fn rewound(&self) -> Option<Box<dyn Iter>> {
        None
    }
    /// Returns true on succes and false otherwise
    fn rewind(&mut self) -> bool {
        false
    }
    /// Get the exact number of remaining elements or None if not applicable
    fn remaining(&self) -> Option<usize> {
        None
    }
    /// Get the number of already consumed elements or None if not applicable
    fn pos(&self) -> Option<usize> {
        None
    }
    fn reverse(&mut self) -> bool {
        false
    }
    fn reversed(&self) -> Option<Box<dyn Iter>> {
        None
    }
    fn acquire(&self) -> Box<dyn Iter>;
    fn consume(self) -> Box<dyn Iter>;
}

pub enum IterKind {
    /// A handle referencing its backing data
    Handle,
    /// This iter owns its data
    Owner,
    /// This iter owns its data and consumes it on use
    ConsumingIter,
}
