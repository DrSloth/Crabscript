use crate::{
    base::{Args, DayObject, IterHandle},
    iter::range::RangeIter,
};

/*
pub fn iter(mut args: Args) -> DayObject {
    match args.remove(0) {
        DayObject::Array(arr) => DayObject::Iter(DayIterator(Arc::new(Box::new(arr.into_iter())))),
        _ => panic!(),
    }
}*/

/*
//IMPORTANT The Crabscript documentation has to clarify this extremely
//everything in crabscript is by default clone/copy thats the general
//semantics of the language, that means the Iter would be cloned into the
//next function and next on that would be called, the next method thus only 
//makes sense if passed as ref.
//It probably would be best if next would be a macro macros in generall help a lot with efficiency.

//Or as this is a language construct it gets special treatment and it gets called with ()
pub fn next(mut args: Args) -> DayObject {
    match args.remove(0) {
        DayObject::Iter(mut i) => i.0.next().unwrap_or(DayObject::None),
        _ => panic!("next needs iter")
    }
}*/

pub fn range(mut args: Args) -> DayObject {
    match (args.remove(0), args.remove(0)) {
        (DayObject::Integer(a), DayObject::Integer(b)) => {
            DayObject::Iter(IterHandle::new(Box::new(RangeIter::new(a, b))))
        }
        _ => panic!("Range creation error (wrong args)"),
    }
}
