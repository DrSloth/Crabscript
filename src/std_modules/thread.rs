use crate::{
    base::{Args, DayObject},
    variables::Variables,
};
use std::{sync::Arc, thread, time::Duration};

//NOTE This is only a prototype the inner workings will change
pub fn sleep(mut args: Args) -> DayObject {
    if let DayObject::Integer(i) = args.remove(0) {
        thread::sleep(Duration::from_millis(i as u64));
    }
    DayObject::None
}

pub fn spawn(mut args: Args, var_mgr: Arc<Variables>) -> DayObject {
    DayObject::Thread {
        id: var_mgr.spawn_thread(args.remove(0), args),
        raw: false,
    }
}

pub fn raw_spawn(mut args: Args, var_mgr: Arc<Variables>) -> DayObject {
    DayObject::Thread {
        id: var_mgr.spawn_thread(args.remove(0), args),
        raw: true,
    }
}

pub fn join(mut args: Args, var_mgr: Arc<Variables>) -> DayObject {
    let mut results = Vec::with_capacity(args.len());
    while args.len() > 0 {
        match args.remove(0) {
            DayObject::Thread { id, raw: _ } => results.push(Arc::clone(&var_mgr).join_thread(id)),
            _ => panic!("Insert an error here"),
        }
    }

    if results.len() == 1 {
        results.remove(0)
    } else if results.len() != 0 {
        DayObject::Array(results)
    } else {
        panic!("Return Error here")
    }
}
