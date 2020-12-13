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

///Returns true if all args are falsy
pub fn not(args: Args) -> DayObject {
    let mut b = true;

    for a in args {
        b = b & to_bool_inner(a);
        if b {
            break;
        }
    }

    DayObject::Bool(b)
}

pub fn xor(args: Args) -> DayObject {
    let mut b = false;

    for a in args {
        b = b ^ to_bool_inner(a);
    }

    DayObject::Bool(b)
}
