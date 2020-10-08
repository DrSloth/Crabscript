#[macro_use]
mod dbg_print {
    macro_rules! dbg_print {
        ($arg: expr) => {
            #[cfg(feature = "debug")]
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
mod std_modules;
mod variables;

#[cfg(test)]
mod tests;

pub mod parser;
pub mod tokenizer;

use base::{DayFunction, DayObject};
use node::NodePurpose;
use std::sync::Arc;

macro_rules! add_fn {
    ($mgr:expr, $module_name: ident, $fnname: ident, $fname: literal) => {
        $mgr.populate_const(
            $fname.to_string(),
            DayObject::Function(DayFunction::Function(Arc::new(
                std_modules::$module_name::$fnname,
            ))),
        );
    };
}

macro_rules! add_inst {
    ($mgr:expr, $module_name: ident, $fnname: ident, $fname: literal) => {
        $mgr.populate_const(
            $fname.to_string(),
            DayObject::Function(DayFunction::Instruction(Arc::new(
                std_modules::$module_name::$fnname,
            ))),
        );
    };
}

///Builds the varmgr with the standard functions
pub fn build_varmgr<'a>() -> Arc<variables::Variables<'a>> {
    //variable handler temporarily defined here
    let varmgr = Arc::new(variables::Variables::new());

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

    add_fn!(varmgr, bool_ops, or, "or");
    add_fn!(varmgr, bool_ops, xor, "xor");
    add_fn!(varmgr, bool_ops, and, "and");
    add_fn!(varmgr, bool_ops, not, "not");

    add_fn!(varmgr, comparison, eq, "eq");
    add_fn!(varmgr, comparison, neq, "neq");

    add_fn!(varmgr, array, array, "array");
    add_inst!(varmgr, array, for_each, "for_each");

    add_fn!(varmgr, sys, panic, "panic");

    //add_fn!(varmgr, iter, iter, "iter");

    varmgr
}

pub fn run(src: &str) {
    let varmgr = build_varmgr();

    let lexer = tokenizer::build_lexer().unwrap();

    let tokens = lexer.tokens(src);

    //let tokens = lexer.tokens("println(add(3, 4))");
    let (root_node, _) = parser::parse(tokenizer::TokenStream::new(tokens), NodePurpose::TopLevel);
    dbg_print!(&root_node);

    let varmgr = Arc::new(varmgr);
    for n in root_node {
        n.execute(Arc::clone(&varmgr));
    }
}
