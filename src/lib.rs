mod base;

mod arithmetics;
mod conversion;
mod fs;
mod io;
mod variables;

pub mod parser;
pub mod tokenizer;

use base::{DayFunction, DayObject};
use std::rc::Rc;

macro_rules! add_fn {
    ($mgr:expr, $module_name: ident, $fnname: ident, $fname: literal) => {
        $mgr.def_var(
            $fname.to_string(),
            DayObject::Function(DayFunction::Closure(Rc::new($module_name::$fnname))),
        );
    };
}

///Builds the varmgr with the standard functions
pub fn build_varmgr() -> variables::Variables {
    //variable handler temporarily defined here
    let mut varmgr = variables::Variables::new();

    add_fn!(varmgr, io, print, "print");
    add_fn!(varmgr, io, println, "println");
    add_fn!(varmgr, io, read, "read");
    add_fn!(varmgr, io, readln, "readln");

    add_fn!(varmgr, arithmetics, add, "add");
    add_fn!(varmgr, arithmetics, sub, "sub");
    add_fn!(varmgr, arithmetics, div, "div");
    add_fn!(varmgr, arithmetics, mul, "mul");
    add_fn!(varmgr, arithmetics, modu, "mod");

    add_fn!(varmgr, fs, cat, "cat");
    add_fn!(varmgr, fs, rm, "rm");
    add_fn!(varmgr, fs, touch, "touch");
    add_fn!(varmgr, fs, mv, "mv");
    add_fn!(varmgr, fs, fwrite, "fwrite");

    add_fn!(varmgr, conversion, to_string, "to_string");
    add_fn!(varmgr, conversion, to_int, "to_int");
    add_fn!(varmgr, conversion, to_float, "to_float");
    add_fn!(varmgr, conversion, to_bool, "to_bool");

    varmgr
}

pub fn run() {
    let mut varmgr = build_varmgr();

    let file_content = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    
    let lexer = tokenizer::build_lexer().unwrap();
    //let tokens = lexer.tokens("println(add(3, 4))");
    let tokens = lexer.tokens(&file_content);
    let nodes = parser::parse(tokens);
    dbg!(&nodes);
    for n in nodes {
        n.execute(&mut varmgr);
    }
}
