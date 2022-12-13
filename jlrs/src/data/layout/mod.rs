//! Field and type layouts of Julia data.

pub mod bool;
pub mod char;
#[cfg(feature = "f16")]
pub mod f16;
pub mod foreign;
pub mod inline_layout;
pub mod nothing;
#[cfg(feature = "internal-types")]
pub mod ssa_value;
pub mod tuple;
pub mod union;
pub mod valid_layout;
