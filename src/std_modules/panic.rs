use crate::base::{Args, DayObject};
use super::conversion::to_bool_inner;

/// This is just used for easy errors that propagated through the rust system
/// mainly used for debug purposes
pub fn panic(args: Args) -> DayObject {
    panic!("{:?}", args)
}

/// This is just used for easy errors that propagated through the rust system
/// mainly used for debug purposes
pub fn assert(mut args: Args) -> DayObject {
    assert!(to_bool_inner(args.remove(0)));
    DayObject::None
}

