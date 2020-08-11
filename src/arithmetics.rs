use crate::base::{DayObject::{*, self}, Args};

macro_rules! def_op {
    ($name: ident, $othername: ident, $op: tt) => {
        pub fn $othername(a: &DayObject, b: &DayObject) -> DayObject {
            match (a,b) {
                (Integer(ref a),Integer(ref b)) => Integer(a $op b),
                (Float(ref a),Integer(ref b)) => Float(a $op *b as f64),
                (Integer(ref a),Float(ref b)) => Float(*a as f64 $op b),
                _ => panic!("can only add float and int")
            }
        }
        
        pub fn $name(args: Args) -> DayObject {
            let result = Integer(0);
            for a in args {
                $othername(&result, &a);
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
