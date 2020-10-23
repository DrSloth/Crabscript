use super::conversion::to_string_inner;
use crate::base::{Args, DayObject};
use std::{io::Read, sync::Arc};
use crate::variables::ExecutionManager;

pub fn print(args: Args, _mgr: &Arc<ExecutionManager>) -> DayObject {
    for a in args {
        //dbg_print!(&a);
        print!("{}", to_string_inner(&a))
    }

    DayObject::None
}

pub fn println(args: Args, _mgr: &Arc<ExecutionManager>) -> DayObject {
    if args.len() == 0 {
        println!("");
        return DayObject::None;
    }

    for a in args {
        println!("{}", to_string_inner(&a))
    }

    DayObject::None
}

pub fn readln(args: Args, _mgr: &Arc<ExecutionManager>) -> DayObject {
    if !args.is_empty() {
        eprintln!("read expected 0 argument(s) received: {}", args.len());
    }

    let mut s = String::new();
    std::io::stdin()
        .read_line(&mut s)
        .expect("Unable to read from stding");

    DayObject::Str(s)
}

pub fn read(args: Args, _mgr: &Arc<ExecutionManager>) -> DayObject {
    if !args.is_empty() {
        eprintln!("read expected 0 argument(s) received: {}", args.len());
    }

    DayObject::Character(
        std::io::stdin()
            .bytes()
            .next()
            .expect("Can't read from stdin")
            .expect("Can't read a byte from stdin") as char,
    )
}
