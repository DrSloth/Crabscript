use crate::{
    base::{
        Args,
        DayObject::{self, *},
    },
    variables::ExecutionManager,
};
use std::sync::Arc;

macro_rules! def_op {
    ($name: ident, $othername: ident, $op: tt) => {
        pub fn $othername(a: DayObject, b: DayObject) -> DayObject {
            match (a,b) {
                (Integer(a),Integer(b)) => Integer(a $op b),
                (Float(a),Float(b)) => Float(a $op b),
                (Float(a),Integer(b)) => Float(a $op b as f64),
                (Integer(a),Float(b)) => Float(a as f64 $op b),
                _ => panic!("can only add float and int")
            }
        }

        pub fn $name(mut args: Args, __mgr: &Arc<ExecutionManager>) -> DayObject {
            let mut result = args.remove(0);
            for a in args {
                result = $othername(result, a);
            }

            result
        }
    };
}

def_op!(add, add_two, +);
def_op!(sub, sub_two, -);
def_op!(mul, mul_two, *);
def_op!(div, div_two, /);
def_op!(modu, modu_two, %);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn arithmetics_simple_add() {
        assert_eq!(
            add(
                vec![Integer(10), Integer(20)],
                &Arc::new(ExecutionManager::new())
            ),
            Integer(30)
        )
    }

    #[test]
    fn arithmetics_addf() {
        assert_eq!(
            add(
                vec![Integer(10), Float(20.25)],
                &Arc::new(ExecutionManager::new())
            ),
            Float(30.25)
        )
    }

    #[test]
    fn arithmetics_add_multiple() {
        assert_eq!(
            add(
                vec![Integer(10), Integer(20), Integer(30)],
                &Arc::new(ExecutionManager::new())
            ),
            Integer(60)
        )
    }

    #[test]
    fn arithmetics_add_many() {
        assert_eq!(
            add(
                vec![
                    Integer(10),
                    Float(20.25),
                    Float(1.75),
                    Float(0.42),
                    Integer(10)
                ],
                &Arc::new(ExecutionManager::new())
            ),
            Float(42.42)
        )
    }

    #[test]
    fn arithmetics_simple_sub() {
        assert_eq!(
            sub(
                vec![Integer(10), Integer(20)],
                &Arc::new(ExecutionManager::new())
            ),
            Integer(-10)
        )
    }

    #[test]
    fn arithmetics_subf() {
        assert_eq!(
            sub(
                vec![Integer(10), Float(20.25)],
                &Arc::new(ExecutionManager::new())
            ),
            Float(-10.25)
        )
    }

    #[test]
    fn arithmetics_sub_multiple() {
        assert_eq!(
            sub(
                vec![Integer(100), Integer(20), Integer(30)],
                &Arc::new(ExecutionManager::new())
            ),
            Integer(50)
        )
    }

    #[test]
    fn arithmetics_simple_mul() {
        assert_eq!(
            mul(
                vec![Integer(2), Integer(3)],
                &Arc::new(ExecutionManager::new())
            ),
            Integer(6)
        )
    }

    #[test]
    fn arithmetics_mulf() {
        assert_eq!(
            mul(
                vec![Integer(10), Float(0.2)],
                &Arc::new(ExecutionManager::new())
            ),
            Float(2.0)
        )
    }

    #[test]
    fn arithmetics_mul_multiple() {
        assert_eq!(
            mul(
                vec![Integer(10), Integer(2), Integer(3)],
                &Arc::new(ExecutionManager::new())
            ),
            Integer(60)
        )
    }

    #[test]
    fn arithmetics_simple_div() {
        assert_eq!(
            div(
                vec![Integer(10), Integer(5)],
                &Arc::new(ExecutionManager::new())
            ),
            Integer(2)
        )
    }

    #[test]
    fn arithmetics_divf() {
        assert_eq!(
            div(
                vec![Integer(5), Float(2.0)],
                &Arc::new(ExecutionManager::new())
            ),
            Float(2.5)
        )
    }

    #[test]
    fn arithmetics_div_multiple() {
        assert_eq!(
            div(
                vec![Integer(10), Integer(2), Float(2.0)],
                &Arc::new(ExecutionManager::new())
            ),
            Float(2.5)
        )
    }

    #[test]
    fn arithmetics_simple_mod() {
        assert_eq!(
            modu(
                vec![Integer(10), Integer(5)],
                &Arc::new(ExecutionManager::new())
            ),
            Integer(0)
        )
    }

    #[test]
    fn arithmetics_modf() {
        assert_eq!(
            modu(
                vec![Integer(5), Float(2.0)],
                &Arc::new(ExecutionManager::new())
            ),
            Float(1.0)
        )
    }

    #[test]
    fn arithmetics_mod_multiple() {
        assert_eq!(
            modu(
                vec![Integer(15), Integer(4), Float(2.0)],
                &Arc::new(ExecutionManager::new())
            ),
            Float(1.0)
        )
    }
}
