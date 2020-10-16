use crate::{
    base::{Args, DayObject, IterHandle},
    variables::Variables,
    std_modules::conversion::to_arr_inner,
};
use std::sync::Arc;

pub use crate::iter::map::map;
pub use crate::iter::range::range;

use crate::iter::arr_iter::arr_iter;

pub fn foreach(mut args: Args, vars: Arc<Variables>) -> DayObject {
    match (args.remove(0), args.remove(0)) {
        (DayObject::Iter(mut iter), DayObject::Function(fun)) => {
            if args.len() > 0 {
                let mut arr = to_arr_inner(vec![args.remove(0)]);
                    arr.insert(0, DayObject::None);
                    while let Some(n) = iter.0.next(Arc::clone(&vars)) {
                        arr[0] = n;
                        fun.call(arr.clone(), Arc::clone(&vars));
                    }
            } else {
                while let Some(n) = iter.0.next(Arc::clone(&vars)) {
                    fun.call(vec![n], Arc::clone(&vars));
                }
            }

            DayObject::None
        }
        _ => panic!(),
    }
}

pub fn iter(mut args: Args) -> DayObject {
    match args.remove(0) {
        DayObject::Array(arr) => DayObject::Iter(IterHandle::new(Box::new(arr_iter(arr)))),
        _ => panic!("iter only accepts array"),
    }
}

pub fn rewind(mut args: Args) -> DayObject {
    match args.remove(0) {
        DayObject::Iter(mut it) => {
            it.0.rewind();
            DayObject::Iter(it)
        }
        _ => panic!("iter only accepts array"),
    }
}

pub fn reverse(mut args: Args) -> DayObject {
    match args.remove(0) {
        DayObject::Iter(mut it) => {
            it.0.reverse();
            DayObject::Iter(it)
        }
        _ => panic!("iter only accepts array"),
    }
}

pub fn collect(mut args: Args, vars: Arc<Variables>) -> DayObject {
    match args.remove(0) {
        DayObject::Iter(mut it) => {
            let mut arr = if let Some(len) = it.0.remaining() {
                Vec::with_capacity(len)
            } else {
                vec![]
            };

            while let Some(data) = it.0.next(vars.get_new_scope()) {
                arr.push(data);
            }

            DayObject::Array(arr)
        }
        _ => panic!("collect needs iter"),
    }
}
