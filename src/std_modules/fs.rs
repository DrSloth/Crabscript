use crate::base::{Args, DayObject};
use std::fs as fio;
use DayObject::*;
use std::sync::Arc;
use crate::variables::ExecutionManager;

pub fn cat(args: Args, _mgr: &Arc<ExecutionManager>) -> DayObject {
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

pub fn touch(args: Args, _mgr: &Arc<ExecutionManager>) -> DayObject {
    for p in args {
        match p {
            Str(s) => {
                fio::File::create(s).expect("error reading file");
            }
            _ => panic!("touch expects only strings as file path"),
        }
    }

    DayObject::None
}

pub fn rm(args: Args, _mgr: &Arc<ExecutionManager>) -> DayObject {
    for p in args {
        match p {
            Str(s) => {
                fio::remove_file(s).expect("error reading file");
            }
            _ => panic!("rm expects only strings as file path"),
        }
    }

    DayObject::None
}

pub fn mv(args: Args, _mgr: &Arc<ExecutionManager>) -> DayObject {
    match &args[..] {
        [Str(from), Str(to)] => fio::rename(from, to).expect("Can't rename file"),
        _ => panic!(
            "mv expects 2 arguments (from: string, to: string) received {:?}",
            args
        ),
    }

    DayObject::None
}

pub fn fwrite(args: Args, _mgr: &Arc<ExecutionManager>) -> DayObject {
    match &args[..] {
        [Str(path), Str(content)] => fio::write(path, content).expect("Can't rename file"),
        _ => panic!(
            "fwrite expects 2 arguments (path: string, content: string) received {:?}",
            args
        ),
    }

    DayObject::None
}
