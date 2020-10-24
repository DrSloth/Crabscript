#[macro_use]
mod dbg_print {
    macro_rules! dbg_print {
        ($arg: expr) => {
            #[cfg(feature = "debug")]
            dbg!($arg);
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

mod base;
pub mod iter;
mod node;
mod std_modules;
mod variables;

pub use variables::hash;

#[cfg(test)]
mod tests;

pub mod parser;
pub mod tokenizer;
mod entry;

pub use entry::run;
