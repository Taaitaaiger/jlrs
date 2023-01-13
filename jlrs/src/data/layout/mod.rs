//! Field and type layouts of Julia data.

macro_rules! impl_ccall_arg {
    ($ty:ident) => {
        unsafe impl $crate::convert::ccall_types::CCallArg for $ty {
            type CCallArgType = Self;
            type FunctionArgType = Self;
        }
    };
}

pub mod bool;
pub mod char;
#[cfg(feature = "f16")]
pub mod f16;
pub mod foreign;
pub mod matching_layout;
pub mod nothing;
#[cfg(feature = "internal-types")]
pub mod ssa_value;
pub mod tuple;
pub mod union;
pub mod valid_layout;
