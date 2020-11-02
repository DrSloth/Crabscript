use crate::{
    base::{Args, DayFunction, DayObject, IterHandle},
    iter::{Iter, IterKind},
    std_modules::conversion::to_arr_inner,
};

use std::{
    fmt::{Debug, Formatter, Result as FmtRes},
    sync::Arc,
};

pub fn map(mut args: Args) -> DayObject {
    match (args.remove(0), args.remove(0)) {
        (DayObject::Iter(inner), DayObject::Function(action)) => {
            DayObject::Iter(IterHandle::new(Box::new(MapIter {
                action,
                inner: inner.0,
                args: if args.len() > 0 {
                    Some(Arc::new(to_arr_inner(vec![args.remove(0)])))
                } else {
                    None
                },
            })))
        }
        _ => panic!("map creation error"),
    }
}

pub struct MapIter {
    action: DayFunction,
    inner: Box<dyn Iter>,
    args: Option<Arc<Vec<DayObject>>>,
}

impl Iter for MapIter {
    fn next(&mut self) -> Option<DayObject> {
        if let Some(data) = self.inner.next() {
            Some(self.action.call(if let Some(arr) = self.args.clone() {
                let mut args = (*arr).clone();
                args.insert(0, DayObject::None);
                args[0] = data;
                args
            } else {
                vec![data]
            }))
        } else {
            None
        }
    }

    fn get_indexed(&self, index: usize) -> Option<DayObject> {
        if let Some(data) = self.inner.get_indexed(index) {
            Some(self.action.call(if let Some(arr) = self.args.clone() {
                let mut args = (*arr).clone();
                args.insert(0, DayObject::None);
                args[0] = data;
                args
            } else {
                vec![data]
            }))
        } else {
            None
        }
    }

    fn kind(&self) -> IterKind {
        IterKind::Handle
    }

    fn acquire(&self) -> Box<dyn Iter> {
        let inner = self.inner.acquire();
        Box::new(Self {
            inner,
            action: self.action.clone(),
            args: self.args.clone(),
        })
    }

    /* fn consume(self: Box<Self>) -> Box<dyn Iter> {
        let inner = self.inner.consume();
        Box::new(Self {
            inner,
            action: self.action.clone()
        })
    } */

    /// Returns a rewound version of this iterator
    fn rewound(&self) -> Option<Box<dyn Iter>> {
        if let Some(inner) = self.inner.rewound() {
            Some(Box::new(Self {
                inner,
                action: self.action.clone(),
                args: self.args.clone(),
            }))
        } else {
            None
        }
    }
    /// Returns true on succes and false otherwise
    fn rewind(&mut self) -> bool {
        self.inner.rewind()
    }
    /// Get the exact number of remaining elements or None if not applicable
    fn remaining(&self) -> Option<usize> {
        self.inner.remaining()
    }
    /// Get the number of already consumed elements or None if not applicable
    fn pos(&self) -> Option<usize> {
        self.inner.pos()
    }
    fn reverse(&mut self) -> bool {
        self.inner.reverse()
    }
    fn reversed(&self) -> Option<Box<dyn Iter>> {
        if let Some(inner) = self.inner.reversed() {
            Some(Box::new(Self {
                inner,
                action: self.action.clone(),
                args: self.args.clone(),
            }))
        } else {
            None
        }
    }
}

impl Debug for MapIter {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtRes {
        write!(f, "MapIter")
    }
}
