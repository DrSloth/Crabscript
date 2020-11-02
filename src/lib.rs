#[macro_use]
mod dbg_print {
    macro_rules! dbg_print {
        ($arg: expr) => {
            #[cfg(feature = "debug")]
            dbg!($arg);
        };
    }

    macro_rules! dbg_print_pretty {
        ($arg: expr) => {
            #[cfg(feature = "debug")]
            println!("{}|{}: {:#?}", line!(), file!(), $arg);
        };
    }

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
