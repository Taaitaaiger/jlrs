//! Mappings between Rust and Julia types for codegen purposes.
//!
//! Functions that are exposed to Julia with the [`julia_module`] macro are implemented basically
//! like this:
//!
//! ```julia
//! function fn_name(arg1::FnArg1, arg2::FnArg2, ...)::FnRet
//!     ccall(fn_ptr, CCallRet, (CCallArg1, CCallArg2, ...), arg1, arg2, ...)
//! end
//! ```
//!
//! These types depend on the type used in Rust, which must implement [`CCallArg`] or
//! [`CCallReturn`] depending on their position. Both these traits have two associated types that
//! are used to construct the appropriate types to be used in the function and ccall signature
//! respectively.
//!
//! You shoudldn't manually implement these traits, they're automatically implemented
//! derived by `JlrsReflect`.

use super::construct_type::ConstructType;

/// Trait implemented by types that can be used as argument types of Rust functions exposed by the
/// [`julia_module`] macro.
pub unsafe trait CCallArg {
    type CCallArgType: ConstructType;
    type FunctionArgType: ConstructType;
}

/// Trait implemented by types that can be used as return types of Rust functions exposed by the
/// [`julia_module`] macro.
pub unsafe trait CCallReturn {
    // TODO: relax? E.g to enable generating return types like `Union{Module, Nothing}`
    type FunctionReturnType: ConstructType;
    type CCallReturnType: ConstructType;
}

macro_rules! impl_ccall_arg {
    ($type:ty) => {
        unsafe impl CCallArg for $type {
            type CCallArgType = Self;
            type FunctionArgType = Self;
        }

        unsafe impl CCallReturn for $type {
            type FunctionReturnType = Self;
            type CCallReturnType = Self;
        }
    };
}

impl_ccall_arg!(u8);
impl_ccall_arg!(u16);
impl_ccall_arg!(u32);
impl_ccall_arg!(u64);
impl_ccall_arg!(usize);
impl_ccall_arg!(i8);
impl_ccall_arg!(i16);
impl_ccall_arg!(i32);
impl_ccall_arg!(i64);
impl_ccall_arg!(isize);
impl_ccall_arg!(f32);
impl_ccall_arg!(f64);
