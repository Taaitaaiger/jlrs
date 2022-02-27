//! Wrappers for inline types

pub mod bool;
pub mod char;
#[cfg(feature = "internal-types")]
pub mod ssa_value;
pub mod tuple;
pub mod union;
