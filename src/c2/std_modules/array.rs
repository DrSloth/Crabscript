use crate::base::{Args, DayObject};

pub fn array(args: Args) -> DayObject {
    DayObject::Array(args.to_vec())
}

pub fn len(args: Args) -> DayObject {
    //NOTE later on things like this will be implemented with more variadic idioms
    if args.len() == 0 {
        panic!("Error invalid args no args")
    }

    if let DayObject::Array(arr) = &args[0] {
        DayObject::Integer(arr.len() as i64)
    } else {
        panic!("Error invalid args non array args")
    }
}

/// Slice into an array by args[0] = arr args[1] = lowerbound
/// args[2] = upperbound
pub fn slice(args: Args) -> DayObject {
    if args.len() == 0 {
        return DayObject::Array(vec![]);
    }

    match (args.get(0), args.get(1), args.get(2)) {
        (Some(DayObject::Array(arr)), Some(DayObject::Integer(lower)), None) => {
            DayObject::Array(arr[(*lower as usize)..].to_vec())
        }
        (
            Some(DayObject::Array(arr)),
            Some(DayObject::Integer(lower)),
            Some(DayObject::Integer(upper)),
        ) => DayObject::Array(arr[(*lower as usize)..(*upper as usize)].to_vec()),
        _ => panic!("Arg error"),
    }
}

//Push needs ref for it to really make sense/to really mutate the content
pub fn push(args: Args) -> DayObject {
    if let DayObject::Array(arr) = &args[0] {
        let mut arr = arr.clone();
        for e in args.iter().skip(1) {
            arr.push(e.clone())
        }

        DayObject::Array(arr)
    } else {
        panic!("Errorius")
    }
}
