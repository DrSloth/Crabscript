use pgc::{GcObject, Gc};
/// The basic type every variable has, it wraps the actual object data
pub type BaseType = Gc<DayObject>;
/// Arguments taken by any function
pub type Args = Vec<BaseType>;
pub type DayFunction = Box<dyn FnMut(Args) -> BaseType>; 

/// The basic data inside a variable
pub enum DayObject {
    None,
    Float(f64),
    Bool(bool),
    Integer(i64),
    Str(String),
    Character(char),
    Array(Vec<BaseType>),
    Function(DayFunction),
}

unsafe impl Send for DayObject {}
unsafe impl Sync for DayObject {}

unsafe impl GcObject for DayObject {
    fn references(&self) -> Vec<Gc<dyn GcObject>> {
        match self {
            Self::Array(v) => v.iter().map(|r| r.references()).flatten().collect::<Vec<_>>(),  
            _ => vec![],
        }
    }
}
