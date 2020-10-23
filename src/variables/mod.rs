pub mod variables;
pub use variables::*;
//pub mod managed_arena;
//mod global_manager;

//NOTE The spot for this is not final
use ahash::{AHasher, RandomState};
use lazy_static::lazy_static;
use std::hash::{BuildHasher, Hasher, Hash};

pub type GlobalHasherBuilder = RandomState;
pub type GlobalHasher = AHasher;

lazy_static! {
    static ref GLOBAL_HASHER_BUILDER: GlobalHasherBuilder = RandomState::with_seeds(1, 9, 8, 4);
    static ref GLOBAL_HASHER: GlobalHasher = GLOBAL_HASHER_BUILDER.build_hasher();
}

pub fn get_global_hasherbuilder() -> GlobalHasherBuilder {
    GLOBAL_HASHER_BUILDER.clone()
}

pub fn get_global_hasher() -> GlobalHasher {
    GLOBAL_HASHER.clone()
}

pub fn hash(ident: &str) -> u64 {
    let mut hasher = get_global_hasher();
    ident.hash(&mut hasher);
    hasher.finish()
}
