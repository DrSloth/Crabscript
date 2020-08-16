use crate::{base::{Args, DayObject}, conversion::{to_string_inner}};
use std::io::Read;

pub fn print(args: Args) -> DayObject {
    for a in args {
        print!("{}", to_string_inner(&a))
    }
    
    DayObject::None
}

pub fn println(args: Args) -> DayObject {
    for a in args {
        println!("{}", to_string_inner(&a))
    }
    
    DayObject::None
}

pub fn readln(args: Args) -> DayObject {
    if !args.is_empty() {
        eprintln!("read expected 0 argument(s) received: {}", args.len());
    }

    let mut s = String::new();
    std::io::stdin().read_line(&mut s).expect("Unable to read from stding");

    DayObject::Str(s)
}

pub fn read(args: Args) -> DayObject {
    if !args.is_empty() {
        eprintln!("read expected 0 argument(s) received: {}", args.len());
    }

    DayObject::Integer(std::io::stdin()
    .bytes()
    .nth(0)
    .expect("Can't read from stdin")
    .expect("Can't read a byte from stdin") as i64)
}
