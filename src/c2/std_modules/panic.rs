use super::conversion::to_bool_inner;
use crate::base::{Args, DayObject};

/// This is just used for easy errors that propagated through the rust system
/// mainly used for debug purposes
pub fn panic(args: Args) -> DayObject {
    if args.len() == 0 {
        panic!("")
    } else if args.len() != 1 {
        panic!("{:?}", args)
    } else {
        panic!("{:?}", args[0])
    }
}

//TODO Improve output of those boys

/// This is just used for easy errors that propagated through the rust system
/// mainly used for debug purposes
pub fn assert(args: Args) -> DayObject {
    if args.len() > 2 {
        assert!(to_bool_inner(&args[0]), format!("{:?}", args));
    } else if args.len() == 2 {
        assert!(to_bool_inner(&args[0]), format!("{:?}", &args[2]));
    } else {
        assert!(to_bool_inner(&args[0]));
    }

    DayObject::None
}
