use crate::base::{Args, DayObject};
use crate::variables::ExecutionManager;
use std::{env::args as argsv, sync::Arc};

pub fn argv(mut args: Args, _mgr: &Arc<ExecutionManager>) -> DayObject {
    //All those args.remove should be better wrapped with something more safe (or i we use VecDeque)
    if args.len() == 0 {
        return DayObject::Array(argsv().map(DayObject::Str).collect());
    } else if let DayObject::Integer(i) = args.remove(0) {
        DayObject::Str(
            argsv()
                .nth(i as usize)
                .expect("There should be an error here"),
        )
    } else {
        return DayObject::Array(argsv().map(DayObject::Str).collect());
    }
}
