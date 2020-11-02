use super::conversion::to_bool_inner;
use crate::base::{Args, DayObject};
use crate::variables::ExecutionManager;
use std::sync::Arc;

pub fn and(args: Args, _mgr: &Arc<ExecutionManager>) -> DayObject {
    let mut b = true;

    for a in args {
        b = b & to_bool_inner(a);
        if !b {
            break;
        }
    }

    DayObject::Bool(b)
}

pub fn or(args: Args, _mgr: &Arc<ExecutionManager>) -> DayObject {
    let mut b = false;

    for a in args {
        b = b | to_bool_inner(a);
        if b {
            break;
        }
    }

    DayObject::Bool(b)
}

pub fn not(mut args: Args, _mgr: &Arc<ExecutionManager>) -> DayObject {
    assert_eq!(args.len(), 1);
    DayObject::Bool(!to_bool_inner(args.remove(0)))
}

pub fn xor(args: Args, _mgr: &Arc<ExecutionManager>) -> DayObject {
    let mut b = false;

    for a in args {
        b = b ^ to_bool_inner(a);
    }

    DayObject::Bool(b)
}
