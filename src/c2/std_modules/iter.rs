use crate::{
  base::{Args, DayObject, IterHandle},
  //std_modules::conversion::to_arr_inner,
};

//pub use crate::iter::map::map;
pub use crate::iter::range::range;

pub fn to_iter_inner(arg: DayObject) -> IterHandle {
    match arg {
        //DayObject::Array(arr) => IterHandle::new(Box::new(arr_iter(arr))),
        DayObject::Iter(it) => it,
        v => panic!("can't convert {:?} to iter", v),
    }
}

/* use crate::iter::arr_iter::arr_iter;

pub fn foreach(mut args: Args) -> DayObject {
    match (args.remove(0), args.remove(0)) {
        (DayObject::Iter(mut iter), DayObject::Function(fun)) => {
            if args.len() > 1 {
                let mut arr = to_arr_inner(vec![args.remove(0)]);
                arr.insert(0, DayObject::None);
                while let Some(n) = iter.0.next() {
                    arr[0] = n;
                    fun.call(arr.clone());
                }
            } else {
                while let Some(n) = iter.0.next() {
                    fun.call(vec![n]);
                }
            }

            DayObject::None
        }
        (DayObject::Function(fun), DayObject::Iter(mut iter)) => {
            if args.len() > 1 {
                let mut arr = to_arr_inner(vec![args.remove(0)]);
                arr.insert(0, DayObject::None);
                while let Some(n) = iter.0.next() {
                    arr[0] = n;
                    fun.call(arr.clone());
                }
            } else {
                while let Some(n) = iter.0.next() {
                    fun.call(vec![n]);
                }
            }

            DayObject::None
        }
        _ => panic!("Invalid argument for foreach"),
    }
}

pub fn iter(mut args: Args) -> DayObject {
    DayObject::Iter(to_iter_inner(args.remove(0)))
}

pub fn rewind(mut args: Args) -> DayObject {
    match args.remove(0) {
        DayObject::Iter(mut it) => {
            it.0.rewind();
            DayObject::Iter(it)
        }
        v => panic!("can't rewind {:?}", v),
    }
}

pub fn reverse(mut args: Args) -> DayObject {
    match args.remove(0) {
        DayObject::Iter(mut it) => {
            it.0.reverse();
            DayObject::Iter(it)
        }
        v => panic!("can't reverse {:?}", v),
    }
}

pub fn collect(mut args: Args) -> DayObject {
    match args.remove(0) {
        DayObject::Iter(it) => collect_inner(it),
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
 */