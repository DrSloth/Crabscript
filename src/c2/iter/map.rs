use crate::{
    base::{Args, DayFunction, DayObject, IterHandle},
    iter::{Iter, IterKind},
    std_modules::conversion::single_value_to_arr,
};

use std::fmt::{Debug, Formatter, Result as FmtRes};

pub fn map(args: Args) -> DayObject {
    match (&args[0], &args[1]) {
        (DayObject::Iter(inner), DayObject::Function(action)) => {
            DayObject::Iter(IterHandle::new(Box::new(MapIter {
                action: action.clone(),
                inner: inner.clone().0,
                args: if args.len() > 0 {
                    let mut given_args = single_value_to_arr(&args[2]);
                    let mut v = Vec::with_capacity(given_args.len() + 1);
                    v.push(DayObject::None);
                    v.append(&mut given_args);
                    v
                } else {
                    vec![DayObject::None]
                },
            })))
        }
        _ => panic!("map creation error"),
    }
}

pub struct MapIter {
    action: DayFunction,
    inner: Box<dyn Iter>,
    args: Vec<DayObject>,
}

impl Iter for MapIter {
    fn next(&mut self) -> Option<DayObject> {
        if let Some(data) = self.inner.next() {
            self.args[0] = data;
            Some(self.action.call(&self.args))
        } else {
            None
        }
    }

    fn get_indexed(&self, index: usize) -> Option<DayObject> {
        if let Some(data) = self.inner.get_indexed(index) {
            //NOTE This solution is inefficient
            //it could be done with unsafe interior mutability
            let mut args = self.args.clone();
            args[0] = data;
            Some(self.action.call(&self.args))
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
