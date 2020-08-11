use crate::base::{Args, DayObject, self};

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
        eprintln!("expected 1 argument received: {}", args.len());
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
