//! Library root: a plain module with a colocated killing test, and a feature-gated one
//! whose mutants are killed by the crate's integration test.

pub mod core;

#[cfg(feature = "boost")]
pub mod boost;
