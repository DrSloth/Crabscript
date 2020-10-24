#[cfg(not(feature="c2"))]
mod parser;
#[cfg(not(feature="c2"))]
pub use parser::Parser;
#[cfg(feature="c2")]
mod parserc2;
#[cfg(feature="c2")]
pub use parserc2::Parser;
pub mod parsing_error;
