use crate::base::{Args, DayObject};

/// This is just used for easy errors that propagated through the rust system
/// mainly used for debug purposes
pub fn panic(args: Args) -> DayObject {
    panic!("{:?}", args)
}
