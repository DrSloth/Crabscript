use std::rc::Rc;

use crate::node;
use crate::variables::Variables;

/// Arguments taken by any function
pub type Args<'a> = Vec<DayObject<'a>>;

/// The basic data inside a variable
#[derive(Clone, PartialEq)]
pub enum DayObject<'a> {
    //The way of representing/creating none variables could be optimised,
    //currently a None is created through a new None value for simplicity sake, but it could also be
    //static/const like pattern but for this zval containers or some similar pattern might be
    //needed to implement it. This can't be done as a new value replaces the value behind a pointer
    None,
    Float(f64),
    Bool(bool),
    Integer(i64),
    Str(String),
    Character(char),
    Array(Vec<DayObject<'a>>),
    Function(DayFunction<'a>),
}

impl DayObject<'_> {
    pub fn call<'a>(&self, args: Args<'a>) -> DayObject<'a> {
        match self {
            DayObject::Function(f) => f.call(args),
            _ => panic!("Tried to call non function value"),
        }
    }
}

unsafe impl Send for DayObject<'_> {}
unsafe impl Sync for DayObject<'_> {}

impl std::fmt::Debug for DayObject<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use DayObject::*;
        match self {
            None => write!(f, "NONE"),
            Float(fl) => write!(f, "{:?}", fl),
            Integer(i) => write!(f, "{:?}", i),
            Bool(b) => write!(f, "{:?}", b),
            Str(s) => write!(f, "{:?}", s),
            Character(c) => write!(f, "{:?}", c),
            Array(a) => write!(f, "{:?}", a),
            Function(_) => write!(f, "Function"),
        }
    }
}

#[derive(Clone)]
pub enum DayFunction<'a> {
    Closure(Rc<dyn Fn(Args) -> DayObject>),
    RuntimeDef(&'a mut node::RootNode<'a>, Variables<'a>),
    // Will also have a function call node variant
}

impl std::fmt::Debug for DayFunction<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use DayFunction::*;
        match self {
            Closure(_) => write!(f, "Function"),
        }
    }
}

impl PartialEq for DayFunction<'_> {
    fn eq(&self, _other: &Self) -> bool {
        panic!("Currently comparing functions is not supported")
    }
}

impl DayFunction<'_> {
    pub fn call<'a>(&self, args: Args<'a>) -> DayObject<'a> {
        match self {
            DayFunction::Closure(f) => f(args),
            DayFunction::RuntimeDef(block, mut var_manager) => {
                block.execute(&mut var_manager);
                DayObject::None // TODO Add return functionality
            }
        }
    }
}
