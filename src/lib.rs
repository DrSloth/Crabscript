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
pub mod iter;
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
    add_fn!(varmgr, conversion, to_arr, "to_arr");

    add_fn!(varmgr, bool_ops, or, "or");
    add_fn!(varmgr, bool_ops, xor, "xor");
    add_fn!(varmgr, bool_ops, and, "and");
    add_fn!(varmgr, bool_ops, not, "not");

    add_fn!(varmgr, comparison, eq, "eq");
    add_fn!(varmgr, comparison, neq, "neq");
    add_fn!(varmgr, comparison, lt, "lt");
    add_fn!(varmgr, comparison, le, "le");
    add_fn!(varmgr, comparison, gt, "gt");
    add_fn!(varmgr, comparison, ge, "ge");

    add_fn!(varmgr, array, array, "array");
    //add_inst!(varmgr, array, for_each, "for_each");

    add_fn!(varmgr, panic, panic, "panic");
    add_fn!(varmgr, panic, assert, "assert");

    add_fn!(varmgr, iter, range, "range");
    add_fn!(varmgr, iter, map, "map");
    add_fn!(varmgr, iter, iter, "iter");
    add_fn!(varmgr, iter, reverse, "reverse");
    add_fn!(varmgr, iter, rewind, "rewind");
    add_inst!(varmgr, iter, foreach, "foreach");
    add_inst!(varmgr, iter, collect, "collect");

    add_inst!(varmgr, functional, call, "call");
    add_fn!(varmgr, functional, apply, "apply");
    add_inst!(varmgr, functional, chain, "chain");
    add_inst!(varmgr, functional, do_times, "do");

    varmgr
}

pub fn run(src: &str) {
    let lexer = tokenizer::build_lexer().unwrap();

    let tokens = lexer.tokens(src);

    let varmgr = build_varmgr();

    //let tokens = lexer.tokens("println(add(3, 4))");
    let parser = parser::Parser::new();
    let (root_node, _) = parser.parse(tokenizer::TokenStream::new(tokens), NodePurpose::TopLevel);
    dbg_print!(&root_node);

    let varmgr = Arc::new(varmgr);
    for n in root_node {
        n.execute(Arc::clone(&varmgr));
    }
}
