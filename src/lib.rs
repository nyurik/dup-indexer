#![doc = include_str!("../README.md")]

#[cfg(feature = "foldhash")]
pub use foldhash;

#[cfg(feature = "foldhash")]
pub type DefaultHashBuilder = foldhash::fast::RandomState;

#[cfg(not(feature = "foldhash"))]
pub type DefaultHashBuilder = std::collections::hash_map::RandomState;

mod owner;
pub use owner::*;

mod deref;
pub use deref::*;
