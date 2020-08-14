pub mod tokenizer;
mod base;
mod variables;
mod io;
mod arithmetics;
mod fs;
pub mod parser;

use base::DayObject;
use std::rc::Rc;

macro_rules! add_module {
    ($mgr:expr, $module_name: ident, $fnname: ident, $fname: literal) => {
        $mgr.def_var($fname.to_string(), DayObject::Function(Rc::new($module_name::$fnname)));    
    };
}

///Builds the varmgr with the standard functions
pub fn build_varmgr() -> variables::Variables {
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
    add_module!(varmgr, fs, cat, "cat");
    add_module!(varmgr, fs, rm, "rm");
    add_module!(varmgr, fs, touch, "touch");
    add_module!(varmgr, fs, mv, "mv");

    varmgr
}

pub fn run() {
    let varmgr = build_varmgr();
    let lexer = tokenizer::build_lexer().unwrap();
    let tokens = lexer.tokens("print(\"Hello, World!\") ");
    let nodes = parser::parse(tokens);
    dbg!(&nodes);
    for n in nodes {
        n.execute();
    }
}