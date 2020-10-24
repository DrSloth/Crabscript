#[cfg(not(feature="c2"))]
pub mod variables;
#[cfg(not(feature="c2"))]
pub use variables::*;
#[cfg(feature="c2")]
pub mod managerc2;
#[cfg(feature="c2")]
pub use managerc2::*;

//pub mod managed_arena;
//mod global_manager;
