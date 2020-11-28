#[macro_use]
mod dbg_print {
    #[allow(unused_macros)]
    macro_rules! dbg_print {
        ($arg: expr) => {
            #[cfg(feature = "debug")]
            dbg!($arg);
        };
    }

    #[allow(unused_macros)]
    macro_rules! dbg_print_pretty {
        ($arg: expr) => {
            #[cfg(feature = "debug")]
            println!("{}|{}: {:#?}", line!(), file!(), $arg);
        };
    }

    #[allow(unused_macros)]
    macro_rules! expect {
        ($expr:expr => $enum:path | $msg:literal) => {{
            if let $enum(item) = $expr {
                item
            } else {
                panic!($msg)
            }
        }};
    }
}

#[cfg(test)]
mod tests;

#[cfg(not(feature = "c2"))]
mod c1;
#[cfg(not(feature = "c2"))]
pub use c1::*;

#[cfg(feature = "c2")]
mod c2;
#[cfg(feature = "c2")]
pub use c2::*;
