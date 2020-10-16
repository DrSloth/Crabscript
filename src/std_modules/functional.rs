use crate::{
    base::{Args, DayObject},
    variables::Variables,
};
use std::sync::Arc;

//TODO add apply function returning a function that gets all given args appended to the actual args
//then map and foreach could take multiple functions. The current api is cool and all but
//doing the applying with some other fn is better and more ergonomic

pub fn call(mut args: Args, var_mgr: Arc<Variables>) -> DayObject {
    match (args.remove(0), args.remove(0)) {
        (DayObject::Function(fun), DayObject::Array(arr)) => fun.call(arr, var_mgr.new_scope()),
        _ => panic!("apply error"),
    }
}

pub fn do_times(mut args: Args, var_mgr: Arc<Variables>) -> DayObject {
    let times = expect!(args.remove(0) => DayObject::Integer | "Expected int as first arg in do");
    let fun =
        expect!(args.remove(0) => DayObject::Function | "Expected function as second arg in do");
    let fun_args = if args.len() > 0 {
        expect!(args.remove(0) => DayObject::Array | "Expected function as second arg in do")
    } else {
        vec![]
    };

    for _ in 0..times {
        fun.call(fun_args.clone(), Arc::clone(&var_mgr).new_scope());
    }

    DayObject::None
}
