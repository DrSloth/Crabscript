use crate::base::{Args, DayObject};

// TODO implement this with macro

//NOTE should they break if they become false?

macro_rules! cmp_fn {
    ($name: ident, $op: tt) => {
        pub fn $name(args: Args) -> DayObject {
            let mut b = true;

            for a in args.windows(2) {
                b = a[0] == a[1] && b;
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

//Writing some benchmarks could be quite cool to see how fast the language is compared to python (we also need references)
