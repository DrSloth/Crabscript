use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

use crate::variables::Variables;

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
    pub fn call<'a>(&'a self, args: Args, var_manager: Arc<Variables<'a>>) -> DayObject {
        match self {
            DayObject::Function(f) => f.call(args, Arc::clone(&var_manager)),
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

impl Hash for DayObject {
    fn hash<H: Hasher>(&self, state: &mut H) {
        //NOTE Currently hashing functions is not supported (need additional info for this)
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
                use DayFunction::*;
                state.write_u8(7);
                match f {
                    RuntimeDef(i) => state.write_usize(*i),
                    //IMPORTANT I don't know if this really works
                    Function(c) => state.write_usize(c.as_ref() as *const _ as *const () as usize),
                    Instruction(c) => {
                        state.write_usize(c.as_ref() as *const _ as *const () as usize)
                    }
                }
            }
            /*Iter(i) => {
                state.write_u8(7);
                state.write_usize(i.0.as_ref() as *const _ as *const () as usize)
            }*/
        }
    }
}

#[derive(Clone)]
pub enum DayFunction {
    Function(Arc<dyn Fn(Args) -> DayObject>),
    //NOTE I couldn't find a good name
    Instruction(Arc<dyn Fn(Args, Arc<Variables>) -> DayObject>),
    RuntimeDef(usize),
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
            (RuntimeDef(a), RuntimeDef(b)) => a == b,
            (Function(a), Function(b)) => {
                (a.as_ref() as *const dyn Fn(Args) -> DayObject)
                    == (b.as_ref() as *const dyn Fn(Args) -> DayObject)
            }
            _ => false,
        }
    }
}

impl DayFunction {
    pub fn call(&self, args: Args, var_manager: Arc<Variables>) -> DayObject {
        match self {
            DayFunction::Function(f) => f(args),
            DayFunction::Instruction(i) => i(args, var_manager),
            DayFunction::RuntimeDef(id) => var_manager.exec_fn(args, *id),
        }
    }
}
