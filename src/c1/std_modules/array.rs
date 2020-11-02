use crate::base::{Args, DayObject};
use crate::variables::ExecutionManager;
use std::sync::Arc;

pub fn array(args: Args, _mgr: &Arc<ExecutionManager>) -> DayObject {
    DayObject::Array(args)
}

pub fn len(mut args: Args, _mgr: &Arc<ExecutionManager>) -> DayObject {
    //NOTE later on things like this will be implemented with more variadic idioms
    if args.len() == 0 {
        panic!("Error invalid args no args")
    }

    if let DayObject::Array(arr) = args.remove(0) {
        DayObject::Integer(arr.len() as i64)
    } else {
        panic!("Error invalid args non array args")
    }
}

/// Slice into an array by args[0] = arr args[1] = lowerbound
/// args[2] = upperbound
pub fn slice(args: Args, _mgr: &Arc<ExecutionManager>) -> DayObject {
    if args.len() == 0 {
        return DayObject::Array(vec![]);
    }

    match (args.get(0), args.get(1), args.get(2)) {
        (Some(DayObject::Array(arr)), Some(DayObject::Integer(lower)), None) => {
            DayObject::Array(arr[(*lower as usize)..].to_vec())
        }
        (
            Some(DayObject::Array(arr)),
            Some(DayObject::Integer(lower)),
            Some(DayObject::Integer(upper)),
        ) => DayObject::Array(arr[(*lower as usize)..(*upper as usize)].to_vec()),
        _ => panic!("Arg error"),
    }
}

//Push needs ref for it to really make sense/to really mutate the content
pub fn push(mut args: Args, _mgr: &Arc<ExecutionManager>) -> DayObject {
    if let DayObject::Array(mut arr) = args.remove(0) {
        for __ in 0..args.len() {
            arr.push(args.remove(0))
        }

        DayObject::Array(arr)
    } else {
        panic!("Errorius")
    }
}
