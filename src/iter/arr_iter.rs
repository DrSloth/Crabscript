use crate::{base::DayObject, iter::{IterData, Iter, IterKind}};
use std::sync::Arc;

pub struct ArrIterData {
    data: Vec<DayObject>
}

impl IterData for ArrIterData {
    fn acquire(self: Arc<Self>) -> Arc<dyn Iter> {

    }

    fn consume(self) -> Arc<dyn Iter> {
        Arc::new(ArrIter {
            data: self.data
        })
    }

    fn get_indexed(&self, index: usize) -> Option<DayObject> {
        self.data.get(index).map(|x| x.clone())
    }
}

pub struct ArrIter {
    data: Vec<DayObject>,
}

impl Iter for ArrIter {
    
}
