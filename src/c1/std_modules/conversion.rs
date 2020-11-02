use crate::base::{
    Args,
    DayObject::{self, *},
};
use crate::variables::ExecutionManager;
use std::sync::Arc;

impl From<String> for DayObject {
    fn from(s: String) -> Self {
        DayObject::Str(s)
    }
}

impl From<i64> for DayObject {
    fn from(i: i64) -> Self {
        DayObject::Integer(i)
    }
}

impl From<f64> for DayObject {
    fn from(f: f64) -> Self {
        DayObject::Float(f)
    }
}

impl Into<String> for DayObject {
    fn into(self) -> String {
        to_string_inner(&self)
    }
}

impl Into<i64> for DayObject {
    fn into(self) -> i64 {
        match &self {
            Integer(i) => *i,
            Float(f) => *f as i64,
            Str(s) => s
                .trim()
                .parse()
                .unwrap_or_else(|_| panic!("Can't convert {:?} to int", self)),
            _ => panic!("Can't convert {:?} to int", self),
        }
    }
}

impl Into<f64> for DayObject {
    fn into(self) -> f64 {
        match &self {
            Integer(i) => *i as f64,
            Float(f) => *f,
            Str(s) => s
                .trim()
                .parse()
                .unwrap_or_else(|_| panic!("Can't convert {:?} to float", self)),
            _ => panic!("Can't convert {:?} to float", self),
        }
    }
}

impl Into<bool> for DayObject {
    fn into(self) -> bool {
        match &self {
            Integer(i) => *i != 0,
            Float(f) => *f != 0.0,
            Str(s) => !s.is_empty(),
            Bool(b) => *b,
            _ => panic!("Can't convert {:?} to bool", self),
        }
    }
}

pub(crate) fn to_string_inner(obj: &DayObject) -> String {
    match obj {
        DayObject::Str(s) => s.clone(),
        DayObject::Bool(b) => b.to_string(),
        DayObject::Character(c) => c.to_string(),
        DayObject::Integer(i) => i.to_string(),
        DayObject::None => "none".to_string(),
        DayObject::Float(f) => f.to_string(),
        DayObject::Array(arr) => format!("{:?}", arr),
        val => format!("{:?}", val),
    }
}

pub fn to_string(args: Args, _mgr: &Arc<ExecutionManager>) -> DayObject {
    if args.len() != 1 {
        eprintln!("to_string expected 1 argument received: {}", args.len());
        std::process::exit(1);
    }

    DayObject::Str(to_string_inner(&args[0]))
}

pub fn to_int_inner(arg: DayObject) -> i64 {
    arg.into()
}

pub fn to_int(mut args: Args, _mgr: &Arc<ExecutionManager>) -> DayObject {
    if args.len() != 1 {
        panic!("to_int expects exactly one argument")
    }

    DayObject::Integer(to_int_inner(args.remove(0)))
}

pub fn to_float(mut args: Args, _mgr: &Arc<ExecutionManager>) -> DayObject {
    if args.len() != 1 {
        panic!("to_float expects exactly one argument")
    }

    DayObject::Float(args.remove(0).into())
}

pub fn to_bool(mut args: Args, _mgr: &Arc<ExecutionManager>) -> DayObject {
    if args.len() != 1 {
        panic!("to_bool expects exactly one argument")
    }

    DayObject::Bool(to_bool_inner(args.remove(0)))
}

pub fn to_arr(args: Args, _mgr: &Arc<ExecutionManager>) -> DayObject {
    DayObject::Array(to_arr_inner(args))
}

pub fn to_arr_inner(mut args: Args) -> Vec<DayObject> {
    if args.len() == 1 {
        let val = args.remove(0);
        if let DayObject::Array(arr) = val {
            arr
        } else {
            vec![val]
        }
    } else {
        args
    }
}

pub fn to_bool_inner(arg: DayObject) -> bool {
    arg.into()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn conversion_itos() {
        assert_eq!(
            to_string(vec![Integer(10)], &Arc::new(ExecutionManager::new())),
            Str("10".to_string())
        )
    }

    #[test]
    fn conversion_ftos() {
        assert_eq!(
            to_string(vec![Float(10.3333456)], &Arc::new(ExecutionManager::new())),
            Str("10.3333456".to_string())
        )
    }
}
