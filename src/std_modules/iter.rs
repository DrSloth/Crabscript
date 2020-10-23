use crate::{
    base::{Args, DayObject, IterHandle},
    std_modules::conversion::to_arr_inner,
    variables::ExecutionManager,
};
use std::sync::Arc;

pub use crate::iter::map::map;
pub use crate::iter::range::range;

use crate::iter::arr_iter::arr_iter;

pub fn foreach(mut args: Args, mgr: &Arc<ExecutionManager>) -> DayObject {
    match (args.remove(0), args.remove(0)) {
        (DayObject::Iter(mut iter), DayObject::Function(fun)) => {
            if args.len() > 1 {
                let mut arr = to_arr_inner(vec![args.remove(0)]);
                arr.insert(0, DayObject::None);
                while let Some(n) = iter.0.next(mgr) {
                    arr[0] = n;
                    fun.call(arr.clone(), mgr);
                }
            } else {
                while let Some(n) = iter.0.next(mgr) {
                    fun.call(vec![n], mgr);
                }
            }

            DayObject::None
        }
        (DayObject::Function(fun), DayObject::Iter(mut iter)) => {
            if args.len() > 1 {
                let mut arr = to_arr_inner(vec![args.remove(0)]);
                arr.insert(0, DayObject::None);
                while let Some(n) = iter.0.next(mgr) {
                    arr[0] = n;
                    fun.call(arr.clone(), mgr);
                }
            } else {
                while let Some(n) = iter.0.next(mgr) {
                    fun.call(vec![n], mgr);
                }
            }

            DayObject::None
        }
        _ => panic!("Invalid argument for foreach"),
    }
}

pub fn iter(mut args: Args, mgr: &Arc<ExecutionManager>) -> DayObject {
    DayObject::Iter(to_iter_inner(args.remove(0), mgr))
}

pub fn to_iter_inner(arg: DayObject, mgr: &Arc<ExecutionManager>) -> IterHandle {
    match arg {
        DayObject::Array(arr) => IterHandle::new(Box::new(arr_iter(arr, mgr))),
        DayObject::Iter(it) => it,
        v => panic!("can't convert {:?} to iter", v),
    }
}

pub fn rewind(mut args: Args, _mgr: &Arc<ExecutionManager>) -> DayObject {
    match args.remove(0) {
        DayObject::Iter(mut it) => {
            it.0.rewind();
            DayObject::Iter(it)
        }
        v => panic!("can't rewind {:?}", v),
    }
}

pub fn reverse(mut args: Args, _mgr: &Arc<ExecutionManager>) -> DayObject {
    match args.remove(0) {
        DayObject::Iter(mut it) => {
            it.0.reverse();
            DayObject::Iter(it)
        }
        v => panic!("can't reverse {:?}", v),
    }
}

pub fn collect(mut args: Args, mgr: &Arc<ExecutionManager>) -> DayObject {
    match args.remove(0) {
        DayObject::Iter(it) => collect_inner(it, mgr),
        _ => panic!("collect needs iter"),
    }
}

pub fn collect_inner(mut iter: IterHandle, mgr: &Arc<ExecutionManager>) -> DayObject {
    let mut arr = if let Some(len) = iter.0.remaining() {
        Vec::with_capacity(len)
    } else {
        vec![]
    };

    let scope = mgr.get_new_scope();
    while let Some(data) = iter.0.next(&scope) {
        arr.push(data);
        scope.clear_scope_ref();
    }

    DayObject::Array(arr)
}
