//! Library root: a plain module and a feature-gated one.

pub mod core;

#[cfg(feature = "boost")]
pub mod boost;
