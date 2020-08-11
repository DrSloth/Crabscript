mod base;
mod variables;
mod io;
mod arithmetics;

use base::DayObject;
use std::rc::Rc;

pub fn run() {
    //variable handler temporarily defined here
    let mut varmgr = variables::Variables::new();
    varmgr.def_var("print".to_string(), DayObject::Function(Rc::new(io::print)));
}
