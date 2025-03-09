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
//! [`CCallReturn`] depending on their position. Both these traits have several associated types
//! that are used to construct the appropriate types to be used in the function and ccall
//! signatures.
//!
//! You shoudldn't manually implement these traits, they're automatically implemented
//! by `JlrsCore.Reflect` if supported.
//!
//! [`julia_module`]: ::jlrs_macros::julia_module

#[cfg(feature = "ccall")]
use crate::{
    data::managed::module::JlrsCore,
    prelude::{JuliaString, Managed},
};
use crate::{
    data::{managed::value::ValueRet, types::construct_type::ConstructType},
    prelude::{JlrsResult, LocalScope as _, Nothing}, weak_handle_unchecked,
};

/// Trait implemented by types that can be used as argument types of Rust functions exposed by the
/// [`julia_module`] macro.
///
/// [`julia_module`]: ::jlrs_macros::julia_module
pub unsafe trait CCallArg {
    /// Type constructor for the type taken by the generated Julia function.
    type CCallArgType: ConstructType;

    /// Type constructor for the type taken by the `ccall`ed function.
    type FunctionArgType: ConstructType;
}

/// Trait implemented by types that can be used as return types of Rust functions exposed by the
/// [`julia_module`] macro.
///
/// [`julia_module`]: ::jlrs_macros::julia_module
pub unsafe trait CCallReturn {
    /// Type constructor for the type returned by the generated Julia function.
    type FunctionReturnType: ConstructType;

    /// Type constructor for the type returned by the `ccall`ed function.
    type CCallReturnType: ConstructType;

    /// Type returned to Julia after calling `Self::return_or_throw`.
    type ReturnAs: CCallReturn;

    /// Convert `self` to data that can be returned to Julia, or throw an exception.
    ///
    /// You should never need to call this method manually. It is called automatically by the glue
    /// code generated by the `julia_module` macro just before returning to Julia.
    ///
    /// Safety:
    ///
    /// This method must only be called just before returning from Rust to Julia. There must be no
    /// pending drops in any of the Rust frames between Julia and the current frame.
    unsafe fn return_or_throw(self) -> Self::ReturnAs;
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
            type ReturnAs = Self;

            #[inline]
            unsafe fn return_or_throw(self) -> Self::ReturnAs {
                return self;
            }
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

unsafe impl<T: CCallReturn> CCallReturn for Result<T, ValueRet> {
    type FunctionReturnType = T::FunctionReturnType;
    type CCallReturnType = T::CCallReturnType;
    type ReturnAs = T;

    #[inline]
    unsafe fn return_or_throw(self) -> Self::ReturnAs {
        #[cfg(feature = "ccall")]
        {
            match self {
                Ok(t) => t,
                Err(e) => crate::runtime::handle::ccall::throw_exception(e),
            }
        }

        #[cfg(not(feature = "ccall"))]
        unimplemented!(
            "CCallReturn::return_or_throw can only be called if the `ccall` feature is enabled"
        )
    }
}

unsafe impl CCallReturn for () {
    type FunctionReturnType = Nothing;
    type CCallReturnType = Nothing;
    type ReturnAs = ();

    #[inline]
    unsafe fn return_or_throw(self) -> Self::ReturnAs {
        ()
    }
}

unsafe impl<T: CCallReturn> CCallReturn for JlrsResult<T> {
    type FunctionReturnType = T::FunctionReturnType;
    type CCallReturnType = T::CCallReturnType;
    type ReturnAs = T;

    #[inline]
    unsafe fn return_or_throw(self) -> Self::ReturnAs {
        #[cfg(feature = "ccall")]
        {
            match self {
                Ok(t) => t,
                Err(e) => {
                    let handle = weak_handle_unchecked!();
                    let e = handle.local_scope::<1>(
                        |mut frame| {
                            let msg = JuliaString::new(&mut frame, format!("{}", e)).as_value();
                            let err =
                                JlrsCore::jlrs_error(&frame).instantiate_unchecked(&frame, [msg]);
                            err.leak()
                        },
                    );
                    crate::runtime::handle::ccall::throw_exception(e)
                }
            }
        }

        #[cfg(not(feature = "ccall"))]
        unimplemented!(
            "CCallReturn::return_or_throw can only be called if the `ccall` feature is enabled"
        )
    }
}
