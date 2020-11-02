use crate::base::DayObject;

pub mod arr_iter;
pub mod map;
pub mod range;

//TODO make acquire not rewind anymore
//NOTE Against the previous plan iterators are internally clone and
//are not stored in an arena. The solution would rather be iters over references
//and moving.

/// This is used inside the var manager as backing data for iterators
/*pub trait IterData<'a> {
    fn acquire(self: Arc<Self>, data_id: usize) -> Box<dyn Iter>;
    ///Calling consume directly on data is invalid and will most likely panic
    fn consume(self: Arc<Self>) -> Box<dyn Iter>;
    fn get_indexed(&self, _index: usize) -> Option<DayObject> {
        None
    }
}*/

/// A CrabScript iterator
pub trait Iter {
    /// Get the next element of the iter
    fn next(&mut self) -> Option<DayObject>;
    fn get_indexed(&self, index: usize) -> Option<DayObject>;
    /// Get which kind of iter this is
    fn kind(&self) -> IterKind;
    //fn consume(self: Box<Self>) -> Box<dyn Iter>;
    fn acquire(&self) -> Box<dyn Iter>;
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
    /// reversing should also rewind the iterator
    fn reverse(&mut self) -> bool {
        false
    }
    /// reversing should also rewind the iterator
    fn reversed(&self) -> Option<Box<dyn Iter>> {
        None
    }
}

pub enum IterKind {
    /// A handle referencing its backing data
    Handle,
    /// This iter owns its data
    Owner,
    /// This iter owns its data and consumes it on use
    ConsumingIter,
}
