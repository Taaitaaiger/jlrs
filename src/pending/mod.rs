//! Support for sharing data between Rust and Julia.

pub(crate) mod borrowed_array;
pub(crate) mod managed_array;
pub(crate) mod owned_array;
pub mod primitive;
