use crate::{
    base::{Args, DayFunction, DayObject},
    variables::ExecutionManager,
};
use std::sync::Arc;

pub fn noop(_args: Args, _mgr: &Arc<ExecutionManager>) -> DayObject {
    DayObject::None
}

pub fn call(mut args: Args, mgr: &Arc<ExecutionManager>) -> DayObject {
    match (args.remove(0), args.remove(0)) {
        (DayObject::Function(fun), DayObject::Array(arr)) => fun.call(arr, &mgr.get_new_scope()),
        _ => panic!("call error"),
    }
}

pub fn apply(mut args: Args, _mgr: &Arc<ExecutionManager>) -> DayObject {
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

pub fn chain(mut args: Args, mgr: &Arc<ExecutionManager>) -> DayObject {
    if let Some(DayObject::Array(initial_args)) = args.pop() {
        //TODO rethink this len of a at the end is always 1
        //NOTE i don't know if new_scope is correct
        if args.len() == 0 {
            panic!("Return error here")
        } else {
            let mut a = args.remove(0).call(initial_args, &mgr.get_new_scope());
            for f in args {
                a = f.call(vec![a], &mgr.get_new_scope());
            }

            a
        }
    } else {
        //NOTE Maybe the order of the args in this fn should change
        panic!("Expected array as last arg to chain")
    }
}

pub fn chained(args: Args, mgr: &Arc<ExecutionManager>) -> DayObject {
    let exec_mgr = Arc::clone(mgr);
    DayObject::Function(DayFunction::RuntimeDef(mgr.def_rust_func(Arc::new(
        move |initial_args| {
            //NOTE maybe this needs a type check (depends on runtime error handling)
            let mut args = args.clone();
            let mut a = args.remove(0).call(initial_args, &exec_mgr.get_new_scope());
            for f in &args {
                a = f.call(vec![a], &exec_mgr.get_new_scope())
            }

            a
        },
    ))))
}

pub fn do_times(mut args: Args, mgr: &Arc<ExecutionManager>) -> DayObject {
    let times = expect!(args.remove(0) => DayObject::Integer | "Expected int as first arg in do");
    let fun =
        expect!(args.remove(0) => DayObject::Function | "Expected function as second arg in do");
    let fun_args = if args.len() > 0 {
        expect!(args.remove(0) => DayObject::Array | "Expected function args as third arg in do")
    } else {
        vec![]
    };

    let mut results = Vec::with_capacity(times as usize);

    let scope = mgr.get_new_scope();
    for _ in 0..times {
        //This line was very innefficient:
        //results.push(fun.call(fun_args.clone(), &var_mgr.get_new_scope()));
        results.push(fun.call(fun_args.clone(), &scope));
        scope.clear_scope_ref();
    }

    DayObject::Array(results)
}

pub fn repeated(mut args: Args, mgr: &Arc<ExecutionManager>) -> DayObject {
    let times = expect!(args.remove(0) => DayObject::Integer | "Expected int as first arg in do");
    let fun =
        expect!(args.remove(0) => DayObject::Function | "Expected function as second arg in do");
    let fun_args = if args.len() > 0 {
        expect!(args.remove(0) => DayObject::Array | "Expected function args as second arg in do")
    } else {
        vec![]
    };

    for _ in 0..times {
        fun.call(fun_args.clone(), &mgr.get_new_scope());
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
        .unwrap();
    }

    #[test]
    fn func_apply() {
        run("
            let double = apply(mul, 2)
            let eight = double(4)
            assert(eq(eight, 8))
        ")
        .unwrap();
    }

    #[test]
    fn func_apply_chain() {
        run("
            let answer = chain(add, apply(mul, 4), apply(add, 2), array(5,5))
            assert(eq(answer, 42))
        ")
        .unwrap();
    }
}
