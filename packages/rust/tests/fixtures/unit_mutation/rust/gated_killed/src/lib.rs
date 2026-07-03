//! Library root: a plain module and a feature-gated one, each with a
//! mutant-killing inline test.

pub mod core;

#[cfg(feature = "boost")]
pub mod boost;
