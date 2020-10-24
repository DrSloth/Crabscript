#[cfg(not(feature="c2"))]
mod entry;
#[cfg(not(feature="c2"))]
pub use entry::*;
#[cfg(feature="c2")]
mod entryc2;
#[cfg(feature="c2")]
pub use entryc2::*;
