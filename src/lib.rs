mod base;
mod variables;
mod io;
mod arithmetics;
mod fs;

use base::DayObject;
use std::rc::Rc;

macro_rules! add_module {
    ($mgr:expr, $module_name: ident, $fnname: ident, $fname: literal) => {
        $mgr.def_var($fname.to_string(), DayObject::Function(Rc::new($module_name::$fnname)));    
    };
}

pub fn run() {
    //variable handler temporarily defined here
    let mut varmgr = variables::Variables::new();

    add_module!(varmgr, io, print, "print");
    add_module!(varmgr, io, println, "println");
    add_module!(varmgr, io, to_string, "to_string");
    add_module!(varmgr, io, read, "read");
    add_module!(varmgr, io, readln, "readln");
    add_module!(varmgr, arithmetics, add, "add");
    add_module!(varmgr, arithmetics, sub, "sub");
    add_module!(varmgr, arithmetics, div, "div");
    add_module!(varmgr, arithmetics, mul, "mul");
    add_module!(varmgr, arithmetics, modu, "mod");
}
