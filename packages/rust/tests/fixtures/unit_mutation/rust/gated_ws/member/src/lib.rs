//! Library root: a plain module with a colocated killing test, and a feature-gated one
//! whose only test lives in the crate's integration tier.

pub mod core;

#[cfg(feature = "boost")]
pub mod boost;
