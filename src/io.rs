use crate::base::{Args, DayObject};
use std::io::Read;

pub(crate) fn to_string_inner(obj: &DayObject) -> String {
    match obj {
        DayObject::Str(s) => s.clone(),
        DayObject::Bool(b) => b.to_string(),
        DayObject::Character(c) => c.to_string(),
        DayObject::Integer(i) => i.to_string(),
        DayObject::None => "NONE".to_string(),
        DayObject::Float(f) => f.to_string(),
        //DayObject::Array
        _ => "".to_string(),
    }
}

pub fn to_string(args: Args) -> DayObject {
    if args.len() != 1 {
        eprintln!("to_string expected 1 argument received: {}", args.len());
        std::process::exit(1);
    }

    DayObject::Str(to_string_inner(&args[0]))
}

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
