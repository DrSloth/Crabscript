use crate::{
    base::{Args, DayFunction, DayObject},
    variables::Variables,
};
use std::sync::Arc;

pub fn call(mut args: Args, var_mgr: Arc<Variables>) -> DayObject {
    match (args.remove(0), args.remove(0)) {
        (DayObject::Function(fun), DayObject::Array(arr)) => fun.call(arr, var_mgr.new_scope()),
        _ => panic!("call error"),
    }
}

pub fn apply(mut args: Args) -> DayObject {
    DayObject::Function(if let DayObject::Function(fun) = args.remove(0) {
        if args.len() == 1 {
            let arg = args.remove(0);
            if let DayObject::Array(arr) = arg {
                DayFunction::Applicator(Box::new(fun), arr)
            } else {
                DayFunction::Applicator(Box::new(fun), vec![arg])
            }
        } else {
            DayFunction::Applicator(Box::new(fun), args)
        }
    } else {
        panic!("apply error")
    })
}

pub fn chain(mut args: Args, var_mgr: Arc<Variables>) -> DayObject {
    if let Some(DayObject::Array(initial_args)) = args.pop() {
        //TODO rethink this len of a at the end is always 1
        //NOTE i don't know if new_scope is correct
        if args.len() == 0 {
            panic!("Return error here")
        } else {
            let mut a = args.remove(0).call(initial_args, var_mgr.get_new_scope());
            for f in args {
                a = f.call(vec![a], var_mgr.get_new_scope());
            }

            a
        }
    } else {
        //NOTE Maybe the order of the args in this fn should change
        panic!("Expected array as last arg to chain")
    }
}

pub fn chained(args: Args, var_mgr: Arc<Variables>) -> DayObject {
    DayObject::Function(DayFunction::RuntimeDef(Arc::clone(&var_mgr).def_rust_func(
        Arc::new(move |initial_args| {
            //NOTE maybe this needs a type check (depends on runtime error handling)
            let mut args = args.clone();
            let mut a = args.remove(0).call(initial_args, var_mgr.get_new_scope());
            for f in &args {
                a = f.call(vec![a], var_mgr.get_new_scope())
            }

            a
        }),
    )))
}

pub fn do_times(mut args: Args, var_mgr: Arc<Variables>) -> DayObject {
    let times = expect!(args.remove(0) => DayObject::Integer | "Expected int as first arg in do");
    let fun =
        expect!(args.remove(0) => DayObject::Function | "Expected function as second arg in do");
    let fun_args = if args.len() > 0 {
        expect!(args.remove(0) => DayObject::Array | "Expected function as second arg in do")
    } else {
        vec![]
    };

    for _ in 0..times {
        fun.call(fun_args.clone(), Arc::clone(&var_mgr).new_scope());
    }

    DayObject::None
}

#[cfg(test)]
mod functional_tests {
    use crate::run;
    #[test]
    fn func_chain() {
        run("
            fn double {
                ret mul(args[0], 2)
            }

            let six = chain(add, double, array(1,2))
            assert(eq(six, 6))
        ")
    }

    #[test]
    fn func_apply() {
        run("
            let double = apply(mul, 2)
            let eight = double(4)
            assert(eq(eight, 8))
        ")
    }

    #[test]
    fn func_apply_chain() {
        run("
            let answer = chain(add, apply(mul, 4), apply(add, 2), array(5,5))
            assert(eq(answer, 42))
        ")
    }
}
