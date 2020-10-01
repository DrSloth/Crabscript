use crate::base::{Args, DayObject};

pub fn eq(args: Args) -> DayObject {
    let mut b = true;

    for a in args.windows(2) {
        b = a[0] == a[1];
    }

    DayObject::Bool(b)
}

pub fn neq(args: Args) -> DayObject {
    let mut b = true;

    for a in args.windows(2) {
        b = a[0] != a[1];
    }

    DayObject::Bool(b)
}

//NOTE: misses gt, lt, le, ge

//Writing some benchmarks could be quite cool to see how fast the language is compared to python (we also need references)
