pub mod base;
pub mod iter;
pub mod manager;
pub mod node;
pub mod std_modules;

#[cfg(not(feature = "c2"))]
pub use variables::hash;

pub mod parser;
pub mod parsing_error;
pub mod tokenizer;

use ahash::RandomState as AHasherBuilder;
use base::RustFunction;
use parser::Parser;
use std::{collections::HashMap, sync::Arc};
use tokenizer::{build_lexer, TokenStream};

pub type PreMap = HashMap<&'static str, RustFunction, AHasherBuilder>;

macro_rules! add_fn {
    ($map:expr, $module_name: ident, $fnname: ident, $fname: literal) => {
        $map.insert($fname, Arc::new(std_modules::$module_name::$fnname));
    };
}

/// Builds the pre_map with the standard functions
/// this will only be used until modules land, after that
/// this logic might be moved to the parser
pub fn build_pre_map<'a>() -> HashMap<&'static str, RustFunction, AHasherBuilder> {
    //NOTE currently a RandomState hasher is used if it makes sense to use a fixed one it will be used
    let mut pre_map: PreMap = HashMap::with_capacity_and_hasher(54, AHasherBuilder::new());

    add_fn!(pre_map, io, print, "print");
    add_fn!(pre_map, io, println, "println");
    add_fn!(pre_map, io, read, "read");
    add_fn!(pre_map, io, readln, "readln");

    add_fn!(pre_map, arithmetics, add, "add");
    add_fn!(pre_map, arithmetics, sub, "sub");
    add_fn!(pre_map, arithmetics, div, "div");
    add_fn!(pre_map, arithmetics, mul, "mul");
    add_fn!(pre_map, arithmetics, modu, "mod");

    add_fn!(pre_map, fs, cat, "cat");
    add_fn!(pre_map, fs, rm, "rm");
    add_fn!(pre_map, fs, touch, "touch");
    add_fn!(pre_map, fs, mv, "mv");
    add_fn!(pre_map, fs, fwrite, "fwrite");

    add_fn!(pre_map, conversion, to_string, "string");
    add_fn!(pre_map, conversion, to_int, "int");
    add_fn!(pre_map, conversion, to_float, "float");
    add_fn!(pre_map, conversion, to_bool, "bool");
    add_fn!(pre_map, conversion, to_arr, "to_arr");

    add_fn!(pre_map, bool_ops, or, "or");
    add_fn!(pre_map, bool_ops, xor, "xor");
    add_fn!(pre_map, bool_ops, and, "and");
    add_fn!(pre_map, bool_ops, not, "not");

    add_fn!(pre_map, comparison, eq, "eq");
    add_fn!(pre_map, comparison, neq, "neq");
    add_fn!(pre_map, comparison, lt, "lt");
    add_fn!(pre_map, comparison, le, "le");
    add_fn!(pre_map, comparison, gt, "gt");
    add_fn!(pre_map, comparison, ge, "ge");

    add_fn!(pre_map, array, array, "array");
    add_fn!(pre_map, array, len, "len");
    add_fn!(pre_map, array, slice, "slice");
    add_fn!(pre_map, array, push, "push");

    add_fn!(pre_map, panic, panic, "panic");
    add_fn!(pre_map, panic, assert, "assert");

    add_fn!(pre_map, iter, range, "range");
    add_fn!(pre_map, iter, map, "map");
    add_fn!(pre_map, iter, iter, "iter");
    add_fn!(pre_map, iter, reverse, "reverse");
    add_fn!(pre_map, iter, rewind, "rewind");
    add_fn!(pre_map, iter, foreach, "foreach");
    add_fn!(pre_map, iter, collect, "collect");

    add_fn!(pre_map, functional, apply, "apply");
    add_fn!(pre_map, functional, call, "call");
    add_fn!(pre_map, functional, chain, "chain");
    add_fn!(pre_map, functional, chained, "chained");
    add_fn!(pre_map, functional, do_times, "do");
    add_fn!(pre_map, functional, repeated, "repeated");

    add_fn!(pre_map, env, argv, "argv");

    /* add_fn!(pre_map, thread, sleep, "sleep");
    add_fn!(pre_map, thread, spawn, "spawn");
    add_fn!(pre_map, thread, raw_spawn, "raw_spawn");
    add_fn!(pre_map, thread, join, "join"); */

    add_fn!(pre_map, functional, noop, "noop");

    pre_map
}

pub fn run(src: &str) -> Result<(), parsing_error::ParsingError> {
    let lexer = build_lexer().unwrap();

    let tokens = lexer.tokens(src);
    
    let pre_map = build_pre_map();
    let mut parser = Parser::new(pre_map);
    let block = parser.parse_tokens(TokenStream::new(tokens))?;

    block.execute();

    Ok(())
}
