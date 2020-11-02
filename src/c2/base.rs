use crate::node::Block;
use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

/// Arguments taken by any function
pub type Args = Vec<DayObject>;
pub type RustFunction = Arc<dyn Fn(Args) -> DayObject>;
pub type ThreadId = usize;

/// The basic data inside a variable
#[derive(Clone)]
pub enum DayObject {
    None,
    Float(f64),
    Bool(bool),
    Integer(i64),
    Character(char),
    Str(String),
    Array(Vec<DayObject>),
    Function(DayFunction),
    Iter(IterHandle),
    /// This is a handle to a thread in the thread memory arena
    /// if raw is true the thread will not be automatically joined
    Thread {
        id: ThreadId,
        raw: bool,
    },
}

impl DayObject {
    pub fn call(&self, args: Args) -> DayObject {
        match self {
            DayObject::Function(f) => f.call(args),
            _ => panic!("Tried to call non function value"),
        }
    }
}

impl PartialEq for DayObject {
    fn eq(&self, other: &Self) -> bool {
        use DayObject::*;
        match (self, other) {
            (None, None) => true,
            (Float(f1), Float(f2)) => *f1 == *f2,
            (Integer(i1), Float(f2)) => *i1 as f64 == *f2,
            (Float(f1), Integer(i2)) => *f1 == *i2 as f64,
            (Bool(b1), Bool(b2)) => *b1 == *b2,
            (Integer(i1), Integer(i2)) => *i1 == *i2,
            (Str(s1), Str(s2)) => *s1 == *s2,
            (Character(c1), Character(c2)) => *c1 == *c2,
            (Array(a1), Array(a2)) => a1.eq(a2),
            (Function(f1), Function(f2)) => *f1 == *f2,
            (Iter(it1), Iter(it2)) => *it1 == *it2,
            _ => false,
        }
    }
}

use std::cmp::Ordering;

impl PartialOrd for DayObject {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use DayObject::*;
        match (self, other) {
            (None, None) => Some(Ordering::Equal),
            (Float(f1), Float(f2)) => f1.partial_cmp(f2),
            (Integer(i1), Float(f2)) => (*i1 as f64).partial_cmp(f2),
            (Float(f1), Integer(i2)) => (*i2 as f64).partial_cmp(f1),
            (Bool(b1), Bool(b2)) => b1.partial_cmp(b2),
            (Integer(i1), Integer(i2)) => i1.partial_cmp(i2),
            (Str(s1), Str(s2)) => s1.partial_cmp(s2),
            (Character(c1), Character(c2)) => c1.partial_cmp(c2),
            (Array(a1), Array(a2)) => a1.partial_cmp(a2),
            (Function(f1), Function(f2)) => {
                if f1 == f2 {
                    Some(Ordering::Equal)
                } else {
                    Option::None
                }
            }
            (Iter(it1), Iter(it2)) => {
                if it1 == it2 {
                    Some(Ordering::Equal)
                } else {
                    Option::None
                }
            }
            _ => Option::None,
        }
    }
}

unsafe impl Send for DayObject {}
unsafe impl Sync for DayObject {}

impl std::fmt::Debug for DayObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use DayObject::*;
        match self {
            None => write!(f, "none"),
            Float(fl) => write!(f, "{:?}", fl),
            Integer(i) => write!(f, "{:?}", i),
            Bool(b) => write!(f, "{:?}", b),
            Str(s) => write!(f, "{:?}", s),
            Character(c) => write!(f, "{:?}", c),
            Array(a) => write!(f, "{:?}", a),
            Function(_) => write!(f, "Function"),
            Iter(_) => write!(f, "Iter"),
            Thread { id, raw: _ } => write!(f, "Thread(Id: {})", *id),
        }
    }
}

impl Hash for DayObject {
    fn hash<H: Hasher>(&self, state: &mut H) {
        use DayObject::*;
        match self {
            None => state.write_u8(0),
            Integer(i) => {
                state.write_u8(1);
                state.write_i64(*i);
            }
            Float(fl) => {
                state.write_u8(2);
                state.write(&fl.to_ne_bytes());
            }
            Bool(b) => {
                state.write_u8(3);
                state.write_u8(if *b { 1 } else { 0 });
            }
            Str(s) => {
                state.write_u8(4);
                state.write(s.as_bytes());
            }
            Character(c) => {
                state.write_u8(5);
                state.write_u32(*c as u32);
            }
            Array(a) => {
                state.write_u8(6);
                for i in a.iter() {
                    <DayObject as Hash>::hash(i, state)
                }
            }
            Function(f) => {
                state.write_u8(7);
                //TODO Move this to impl Hash for DayFunction
                use DayFunction::*;
                match f {
                    RuntimeDef(n) => state.write_usize(n.as_ref() as *const _ as usize),
                    //IMPORTANT I don't know if this really works
                    Function(c) => state.write_usize(c.as_ref() as *const _ as *const () as usize),
                    Applicator(a, args) => {
                        state.write_usize(a.as_ref() as *const _ as *const () as usize);
                        args.hash(state);
                    }
                }
            }
            Iter(i) => {
                state.write_u8(8);
                state.write_usize(i.0.as_ref() as *const _ as *const () as usize)
            }
            Thread { id, raw: _ } => {
                state.write_u8(9);
                state.write_usize(*id)
            }
        }
    }
}

#[derive(Clone)]
pub enum DayFunction {
    Function(RustFunction),
    Applicator(Box<DayFunction>, Args),
    RuntimeDef(Arc<Block>),
}

impl std::fmt::Debug for DayFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Function")
    }
}

impl PartialEq for DayFunction {
    fn eq(&self, other: &Self) -> bool {
        use DayFunction::*;
        match (self, other) {
            (RuntimeDef(a), RuntimeDef(b)) => a.as_ref() as *const _ == b.as_ref() as *const _,
            (Function(a), Function(b)) => (a.as_ref() as *const _) == (b.as_ref()),
            (Applicator(a, args1), Applicator(b, args2)) => {
                (a.as_ref() as *const _) == (b.as_ref() as *const _) && args1 == args2
            }
            _ => false,
        }
    }
}

impl DayFunction {
    pub fn call(&self, mut args: Args) -> DayObject {
        match self {
            DayFunction::Function(f) => f(args),
            DayFunction::RuntimeDef(block) => block.execute_args(args).value(),
            DayFunction::Applicator(f, apply_args) => {
                let mut a = apply_args.clone();
                args.append(&mut a);

                f.call(args)
            }
        }
    }
}

use crate::iter::Iter;

pub struct IterHandle(pub Box<dyn Iter>);

impl std::fmt::Debug for IterHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Iter")
    }
}

impl IterHandle {
    pub fn new(inner: Box<dyn Iter>) -> Self {
        IterHandle(inner)
    }
}

impl Clone for IterHandle {
    fn clone(&self) -> Self {
        Self(self.0.acquire())
    }
}

impl PartialEq for IterHandle {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ref() as *const _ as *const () == other.0.as_ref() as *const _ as *const ()
    }
}
