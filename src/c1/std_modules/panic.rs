use super::conversion::to_bool_inner;
use crate::base::{Args, DayObject};
use crate::variables::ExecutionManager;
use std::sync::Arc;

/// This is just used for easy errors that propagated through the rust system
/// mainly used for debug purposes
pub fn panic(mut args: Args, _mgr: &Arc<ExecutionManager>) -> DayObject {
    if args.len() == 0 {
        panic!("")
    } else if args.len() != 1 {
        panic!("{:?}", args)
    } else {
        panic!("{:?}", args.remove(0))
    }
}

//TODO Improve output of those boys

/// This is just used for easy errors that propagated through the rust system
/// mainly used for debug purposes
pub fn assert(mut args: Args, _mgr: &Arc<ExecutionManager>) -> DayObject {
    if args.len() > 2 {
        assert!(to_bool_inner(args.remove(0)), format!("{:?}", args));
    } else if args.len() == 2 {
        assert!(
            to_bool_inner(args.remove(0)),
            format!("{:?}", args.remove(0))
        );
    } else {
        assert!(to_bool_inner(args.remove(0)));
    }

    DayObject::None
}
