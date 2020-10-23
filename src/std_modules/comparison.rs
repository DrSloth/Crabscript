use crate::base::{Args, DayObject};
use std::sync::Arc;
use crate::variables::ExecutionManager;

macro_rules! cmp_fn {
    ($name: ident, $op: tt) => {
        pub fn $name(args: Args, _mgr: &Arc<ExecutionManager>) -> DayObject {
            let mut b = true;

            for a in args.windows(2) {
                b = a[0] $op a[1];
                if !b {
                    break
                }
            }

            DayObject::Bool(b)
        }
    };
}

//macros are so fucking cool

cmp_fn!(eq, ==);
cmp_fn!(neq, !=);
cmp_fn!(gt, >);
cmp_fn!(lt, <);
cmp_fn!(ge, >=);
cmp_fn!(le, <=);

//TODO More tests for this

#[cfg(test)]
mod cmp_tests {
    use super::*;
    use crate::base::DayObject;
    #[test]
    fn cmp_eq() {
        assert_eq!(
            eq(vec![DayObject::Integer(10), DayObject::Float(10.0)], &Arc::new(ExecutionManager::new())),
            DayObject::Bool(true)
        )
    }

    #[test]
    fn cmp_eq2() {
        assert_eq!(
            eq(vec![
                DayObject::Integer(10),
                DayObject::Float(10.0),
                DayObject::Integer(10)
            ], &Arc::new(ExecutionManager::new())),
            DayObject::Bool(true)
        )
    }

    #[test]
    fn cmp_neq() {
        assert_eq!(
            neq(vec![
                DayObject::Integer(10),
                DayObject::Str("10".to_string())
            ], &Arc::new(ExecutionManager::new())),
            DayObject::Bool(true)
        )
    }

    #[test]
    fn cmp_gt() {
        assert_eq!(
            gt(vec![
                DayObject::Str("B".to_string()),
                DayObject::Str("A".to_string())
            ], &Arc::new(ExecutionManager::new())),
            DayObject::Bool(true)
        )
    }

    #[test]
    fn cmp_gt2() {
        assert_eq!(
            gt(vec![DayObject::Integer(10), DayObject::Integer(10)], &Arc::new(ExecutionManager::new())),
            DayObject::Bool(false)
        )
    }

    #[test]
    fn cmp_ge() {
        assert_eq!(
            ge(vec![
                DayObject::Integer(10),
                DayObject::Integer(10),
                DayObject::Integer(9),
                DayObject::Integer(9)
            ], &Arc::new(ExecutionManager::new())),
            DayObject::Bool(true)
        )
    }
}

//Writing some benchmarks could be quite cool to see how fast the language is compared to python (we also need references)
