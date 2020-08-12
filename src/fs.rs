use crate::base::{Args, DayObject};
use std::fs as fio;
use DayObject::*;

pub fn cat(args: Args) -> DayObject {
    let mut cated = String::new();
    for p in args {
        match p {
            Str(s) => {
                cated.push_str(&fio::read_to_string(s).expect("error reading file"));
            }
            _ => panic!("cat expects only strings as file path"),
        }
    }

    Str(cated)
}

pub fn touch(args: Args) -> DayObject {
    for p in args {
        match p {
            Str(s) => {
                fio::File::create(s).expect("error reading file");
            }
            _ => panic!("cat expects only strings as file path"),
        }
    }

    DayObject::None
}

pub fn rm(args: Args) -> DayObject {
    for p in args {
        match p {
            Str(s) => {
                fio::remove_file(s).expect("error reading file");
            }
            _ => panic!("cat expects only strings as file path"),
        }
    }

    DayObject::None
}

pub fn mv(args: Args) -> DayObject {
    if args.len() != 2 {
        panic!(
            "fs_rename expects 2 arguments (from: string, to: string) received {}",
            args.len()
        )
    }

    match (args[0], args[1]) {
        (Str(from), Str(to)) => fio::rename(from, to),
        _ => panic!(
            "fs_rename expects 2 arguments (from: string, to: string), type mismatch"
        )
    }

    DayObject::None
}
