use crate::{
    base::DayObject,
    iter::{Iter, IterKind},
    variables::{ExecutionManager, Variables},
};
use std::sync::Arc;

/*pub struct ArrIterData {
    data: Vec<DayObject>
}

impl IterData<'_> for ArrIterData {
    fn acquire(self: Arc<Self>, data_id: usize) -> Box<dyn Iter> {
        Box::new(ArrIterHandle {
            index: 0,
            reverse: false,
            data: Arc::clone(&self),
            data_id
        })
    }

    fn consume(self: Arc<Self>) -> Box<dyn Iter> {
        match Arc::try_unwrap(self) {
            Ok(it) => {
                Box::new(ArrIter {
                    index: 0,
                    reverse: true,
                    data: Arc::new(it.data),
                })
            },
            //NOTE Maybe this will be done differently
            Err(_) => panic!("try to consume non unieque IterData"),
        }
    }

    fn get_indexed(&self, index: usize) -> Option<DayObject> {
        self.data.get(index).map(|x| x.clone())
    }
}*/

pub fn arr_iter(data: Vec<DayObject>, _mgr: &Arc<ExecutionManager>) -> ArrIter {
    ArrIter {
        data: Arc::new(data),
        reverse: false,
        index: 0,
    }
}

#[derive(Clone)]
pub struct ArrIter {
    index: usize,
    reverse: bool,
    data: Arc<Vec<DayObject>>,
    //data_id: usize,
}

impl Iter for ArrIter {
    /// Get the next element of the iter
    fn next(&mut self, _: &Arc<Variables>) -> Option<DayObject> {
        if self.index >= self.data.len() {
            return None;
        }

        let data = if self.reverse {
            self.data.get(self.data.len() - self.index - 1)
        } else {
            self.data.get(self.index)
        };

        if data != None {
            self.index += 1;
        }

        dbg_print!(self.index);
        dbg_print!(data);

        data.map(|val| val.clone())
    }
    fn get_indexed(&self, index: usize, _: &Arc<Variables>) -> Option<DayObject> {
        if self.reverse {
            self.data.get(self.data.len() - index - 1)
        } else {
            self.data.get(index)
        }
        .map(|val| val.clone())
    }
    /// Get which kind of iter this is
    fn kind(&self) -> IterKind {
        IterKind::Owner
    }

    /*   fn consume(self: Box<Self>) -> Box<dyn Iter> {
        self
    }  */

    fn acquire(&self) -> Box<dyn Iter> {
        Box::new(Self {
            index: self.index,
            reverse: self.reverse,
            data: Arc::clone(&self.data),
            //data_id: self.data_id
        })
    }
    /// Returns a rewound version of this iterator
    fn rewound(&self) -> Option<Box<dyn Iter>> {
        let mut clone = self.clone();
        clone.index = 0;
        Some(Box::new(clone))
    }
    /// Returns true on succes and false otherwise
    fn rewind(&mut self) -> bool {
        self.index = 0;
        true
    }
    /// Get the exact number of remaining elements or None if not applicable
    fn remaining(&self) -> Option<usize> {
        Some(self.data.len() - self.index)
    }
    /// Get the number of already consumed elements or None if not applicable
    fn pos(&self) -> Option<usize> {
        Some(self.index)
    }
    fn reverse(&mut self) -> bool {
        self.reverse = !self.reverse;
        self.index = 0;
        true
    }
    fn reversed(&self) -> Option<Box<dyn Iter>> {
        let mut clone = self.clone();
        clone.reverse = !clone.reverse;
        clone.index = 0;

        Some(Box::new(clone))
    }
}
