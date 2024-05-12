//! Traits for converting data.

pub mod ccall_types;
pub mod compatible;
pub mod into_jlrs_result;
pub mod into_julia;
pub mod into_simple_vector;
#[cfg(feature = "jlrs-ndarray")]
pub mod ndarray;
pub mod to_symbol;
pub mod unbox;
