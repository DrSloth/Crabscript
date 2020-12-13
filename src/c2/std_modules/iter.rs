use crate::{
    base::{Args, DayFunction, DayObject, IterHandle},
    std_modules::conversion::{single_value_to_arr, to_arr_inner},
};

pub use crate::iter::arr_iter::arr_iter;
pub use crate::iter::map::map;
pub use crate::iter::range::range;

pub fn to_iter_inner(arg: &DayObject) -> IterHandle {
    match arg {
        DayObject::Array(arr) => IterHandle::new(Box::new(arr_iter(arr.to_vec()))),
        DayObject::Iter(it) => it.clone(),
        v => panic!("can't convert {:?} to iter", v),
    }
}

pub fn foreach_inner(mut iter: IterHandle, fun: &DayFunction, args: Args) -> DayObject {
    let mut arr = to_arr_inner(args);
    arr.insert(0, DayObject::None);
    while let Some(n) = iter.0.next() {
        arr[0] = n;
        fun.call(&arr);
    }

    DayObject::None
}

pub fn foreach(args: Args) -> DayObject {
    match (&args[0], &args[1]) {
        (DayObject::Iter(iter), DayObject::Function(fun)) => {
            foreach_inner(iter.clone(), fun, &single_value_to_arr(&args[2]))
        }
        (DayObject::Function(fun), DayObject::Iter(iter)) => {
            foreach_inner(iter.clone(), fun, &single_value_to_arr(&args[2]))
        }
        _ => panic!("Invalid argument for foreach"),
    }
}

pub fn iter(args: Args) -> DayObject {
    DayObject::Iter(to_iter_inner(&args[0]))
}

pub fn rewind(args: Args) -> DayObject {
    match &args[0] {
        DayObject::Iter(it) => {
            let it = IterHandle(it.0.rewound().expect("Error"));
            DayObject::Iter(it)
        }
        v => panic!("can't rewind {:?}", v),
    }
}

pub fn reverse(args: Args) -> DayObject {
    match &args[0] {
        DayObject::Iter(it) => {
            let it = IterHandle(it.0.reversed().expect("Error"));
            DayObject::Iter(it)
        }
        v => panic!("can't reverse {:?}", v),
    }
}

pub fn collect(args: Args) -> DayObject {
    match &args[0] {
        DayObject::Iter(it) => collect_inner(it.clone()),
        _ => panic!("collect needs iter"),
    }
}

pub fn collect_inner(mut iter: IterHandle) -> DayObject {
    let mut arr = if let Some(len) = iter.0.remaining() {
        Vec::with_capacity(len)
    } else {
        vec![]
    };

    while let Some(data) = iter.0.next() {
        arr.push(data);
    }

    DayObject::Array(arr)
}
