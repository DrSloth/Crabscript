use crate::base::{Args, DayObject};
//use std::sync::Arc;

pub fn array(args: Args) -> DayObject {
    DayObject::Array(args)
}

/*
//NOTE The name of this will either change or the module this is in,
//currently this is just for testing and will change
pub fn for_each(args: Args, vars: Arc<Variables>) -> DayObject {
    let fun = match &args[1] {
        DayObject::Function(fun) => fun,
        _ => panic!("The second a function"),
    };

    match &args[0] {
        DayObject::Array(arr) => {
            for obj in arr {
                fun.call(vec![obj.clone()], Arc::clone(&vars).new_scope());
            }
        }
        _ => panic!("The first arg has to be an array in for_each"),
    }
    DayObject::None
}*/
