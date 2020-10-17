use crate::base::{Args, DayObject};

macro_rules! cmp_fn {
    ($name: ident, $op: tt) => {
        pub fn $name(args: Args) -> DayObject {
            let mut b = true;

            for a in args.windows(2) {
                b = a[0] == a[1];
                if !b {
                    break
                }
            }

            DayObject::Bool(b)
        }
    };
}

//macros are so fucking cool

cmp_fn!(eq, ==);
cmp_fn!(neq, !=);
cmp_fn!(gt, >);
cmp_fn!(lt, <);
cmp_fn!(ge, >=);
cmp_fn!(le, <=);

//TODO Tests for this

//Writing some benchmarks could be quite cool to see how fast the language is compared to python (we also need references)
