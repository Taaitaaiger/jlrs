//! Wrappers for inline types

pub mod bool;
pub mod char;
#[cfg(feature = "f16")]
pub mod f16;
#[cfg(feature = "internal-types")]
pub mod ssa_value;
pub mod tuple;
#[cfg(feature = "jlrs-derive")]
pub mod union;
