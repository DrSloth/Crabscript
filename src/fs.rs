use crate::base::{Args, DayObject};
use std::fs as fio;
use DayObject::*;

pub fn cat(args: Args) -> DayObject {
    let mut cated = String::new();
    for p in args {
        match p {
            Str(s) => {
                cated.push_str(&fio::read_to_string(s).expect("error reading file"));
            },
            _ => panic!("cat expects only strings as file path")
        }
    }

    Str(cated)
}

pub fn create_files(args: Args) -> DayObject {
    for p in args {
        match p {
            Str(s) => {
                fio::File::create(s).expect("error reading file");
            },
            _ => panic!("cat expects only strings as file path"),
        }
    }

    DayObject::None
}
