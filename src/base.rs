use std::rc::Rc;

/// Arguments taken by any function
pub type Args = Vec<DayObject>;

/// The basic data inside a variable
#[derive(Clone, PartialEq)]
pub enum DayObject {
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
    Array(Vec<DayObject>),
    Function(DayFunction),
}

impl DayObject {
    pub fn call(&self, args: Args) -> DayObject {
        match self {
            DayObject::Function(f) => f.call(args),
            _ => panic!("Tried to call non function value"),
        }
    }
}

unsafe impl Send for DayObject {}
unsafe impl Sync for DayObject {}

impl std::fmt::Debug for DayObject {
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
pub enum DayFunction {
    Closure(Rc<dyn Fn(Args) -> DayObject>),
    // Will also have a function call node variant
}

impl std::fmt::Debug for DayFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use DayFunction::*;
        match self {
            Closure(_) => write!(f, "Function"),
        }
    }
}

impl PartialEq for DayFunction {
    fn eq(&self, _other: &Self) -> bool {
        panic!("Currently comparing functions is not supported")
    }
}

impl DayFunction {
    pub fn call(&self, args: Args) -> DayObject {
        match self {
            DayFunction::Closure(f) => f(args),
        }
    }
}
