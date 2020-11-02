use super::conversion::to_bool_inner;
use crate::base::{Args, DayObject};

pub fn and(args: Args) -> DayObject {
    let mut b = true;

    for a in args {
        b = b & to_bool_inner(a);
        if !b {
            break;
        }
    }

    DayObject::Bool(b)
}

pub fn or(args: Args) -> DayObject {
    let mut b = false;

    for a in args {
        b = b | to_bool_inner(a);
        if b {
            break;
        }
    }

    DayObject::Bool(b)
}

pub fn not(mut args: Args) -> DayObject {
    assert_eq!(args.len(), 1);
    DayObject::Bool(!to_bool_inner(args.remove(0)))
}

pub fn xor(args: Args) -> DayObject {
    let mut b = false;

    for a in args {
        b = b ^ to_bool_inner(a);
    }

    DayObject::Bool(b)
}
