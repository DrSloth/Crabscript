use crate::{
    base::DayObject,
    iter::{Iter, IterKind},
};
/*#[derive(Clone, Copy, Debug)]
pub struct RangeIterData {
    dir: Direction,
    low: i64,
    high: i64,
}

impl IterData for RangeIterData {
    fn acquire(self: Arc<Self>) -> Arc<dyn Iter> {
        Arc::new(RangeIter {
            data: (*self).clone(),
            index: 0,
        })
    }

    fn consume(self) -> Arc<dyn Iter> {
        Arc::new(RangeIter {
            data: self,
            index: 0,
        })
    }

    fn get_indexed(&self, index: usize) -> Option<DayObject> {
        use Direction::*;
        match self.dir {
            Positive if (self.low + index as i64) < self.high => Some(DayObject::Integer(self.low + index as i64)),
            Negative if self.high - index as i64 > self.low => Some(DayObject::Integer(self.high - index as i64)),
            _ => None,
        }
    }
}*/

//TODO Range for chars

#[derive(Clone, Debug)]
pub struct RangeIter {
    dir: Direction,
    low: i64,
    high: i64,
    index: usize,
}

impl RangeIter {
    pub fn new(num1: i64, num2: i64) -> Self {
        if num1 < num2 {
            Self {
                low: num1,
                high: num2,
                index: 0,
                dir: Direction::Positive,
            }
        } else {
            Self {
                low: num2,
                high: num1,
                index: 0,
                dir: Direction::Negative,
            }
        }
    }
}

impl Iter for RangeIter {
    fn next(&mut self) -> Option<DayObject> {
        let i = self.index;
        self.index += 1;
        self.get_indexed(i)
    }

    fn kind(&self) -> IterKind {
        IterKind::Owner
    }

    fn rewind(&mut self) -> bool {
        self.index = 0;
        true
    }

    fn rewound(&self) -> Option<Box<dyn Iter>> {
        let mut clone = self.clone();
        clone.index = 0;
        Some(Box::new(clone))
    }

    fn reverse(&mut self) -> bool {
        self.dir = !self.dir;
        self.index = 0;
        true
    }

    fn reversed(&self) -> Option<Box<dyn Iter>> {
        let mut clone = self.clone();
        clone.dir = !self.dir;
        Some(Box::new(clone))
    }

    fn remaining(&self) -> Option<usize> {
        Some((self.high - self.low - self.index as i64) as usize)
    }

    fn pos(&self) -> Option<usize> {
        Some(self.index)
    }

    fn acquire(&self) -> Box<dyn Iter> {
        self.rewound().unwrap()
    }

    fn consume(mut self) -> Box<dyn Iter> {
        self.rewind();
        Box::new(self)
    }

    fn get_indexed(&self, index: usize) -> Option<DayObject> {
        use Direction::*;
        match self.dir {
            Positive if (self.low + index as i64) < self.high => {
                Some(DayObject::Integer(self.low + index as i64))
            }
            Negative if self.high - index as i64 > self.low => {
                Some(DayObject::Integer(self.high - index as i64))
            }
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Positive,
    Negative,
}

impl std::ops::Not for Direction {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Direction::Positive => Direction::Negative,
            Direction::Negative => Direction::Positive,
        }
    }
}
