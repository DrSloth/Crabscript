#[macro_use]
mod dbg_print {
    macro_rules! dbg_print {
        ($arg: expr) => {
            #[cfg(feature="debug")]
            dbg!($arg);
        };
    }

    macro_rules! expect {
        ($expr:expr => $enum:path | $msg:literal) => {{
            if let $enum(item) = $expr {
                item
            } else {
                panic!($msg)
            }
        }};
    }
}

mod base;
mod node;
mod variables;
mod std_modules;

pub mod parser;
pub mod tokenizer;

use base::{DayFunction, DayObject};
use std::rc::Rc;

macro_rules! add_fn {
    ($mgr:expr, $module_name: ident, $fnname: ident, $fname: literal) => {
        $mgr.def_const(
            $fname.to_string(),
            DayObject::Function(DayFunction::Closure(Rc::new(std_modules::$module_name::$fnname))),
        );
    };
}

///Builds the varmgr with the standard functions
pub fn build_varmgr<'a>() -> variables::Variables<'a> {
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

    add_fn!(varmgr, conversion, to_string, "string");
    add_fn!(varmgr, conversion, to_int, "int");
    add_fn!(varmgr, conversion, to_float, "float");
    add_fn!(varmgr, conversion, to_bool, "bool");
    add_fn!(varmgr, conversion, array, "array");

    add_fn!(varmgr, bool_ops, or, "or");
    add_fn!(varmgr, bool_ops, xor, "xor");
    add_fn!(varmgr, bool_ops, and, "and");
    add_fn!(varmgr, bool_ops, not, "not");

    add_fn!(varmgr, comparison, eq, "eq");
    add_fn!(varmgr, comparison, neq, "neq");

    varmgr
}

pub fn run() {
    let mut varmgr = build_varmgr();

    let file_content = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let lexer = tokenizer::build_lexer().unwrap();

    let tokens = lexer.tokens(&file_content);
    
    //let tokens = lexer.tokens("println(add(3, 4))");
    let (root_node, _) = parser::parse(tokenizer::TokenStream::new(tokens));
    dbg_print!(&root_node);
    for mut n in root_node {
        n.execute(&mut varmgr);
    }
}
