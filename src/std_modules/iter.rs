/*use crate::{
    base::{Args, DayIterator, DayObject},
    variables::Variables,
};
use std::sync::Arc;

pub fn iter(mut args: Args) -> DayObject {
    match args.remove(0) {
        DayObject::Array(arr) => DayObject::Iter(DayIterator(Arc::new(Box::new(arr.into_iter())))),
        _ => panic!(),
    }
}*/


