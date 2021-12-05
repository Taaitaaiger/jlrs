//! Convert a `JuliaResult` to a `JlrsResult`.
//!
//! Methods that call the Julia C API and can throw an exception generally return a nested
//! `Result`. The outer error contains no Julia data, while the inner error contains the thrown
//! exception. The `IntoJlrsResult` trait can be used to convert the inner error into the outer
//! error.

use crate::{
    error::{exception, JlrsResult, JuliaResult, CANNOT_DISPLAY_VALUE},
    wrappers::ptr::Wrapper,
};

/// Extension trait that lets you convert a `JuliaResult` to a `JlrsResult`. If an exception
/// is thrown, this method converts the exception to an error message by calling
/// `Base.showerror`.
pub trait IntoJlrsResult<T>: private::IntoJlrsResult {
    /// Convert `self` to `JlrsResult` by calling `Base.showerror` if an exception has been
    /// thrown.
    fn into_jlrs_result(self) -> JlrsResult<T>;
}

impl<T> IntoJlrsResult<T> for JuliaResult<'_, '_, T> {
    fn into_jlrs_result(self) -> JlrsResult<T> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => exception(e.error_string_or(CANNOT_DISPLAY_VALUE)),
        }
    }
}

mod private {
    use crate::error::JuliaResult;

    pub trait IntoJlrsResult {}
    impl<T> IntoJlrsResult for JuliaResult<'_, '_, T> {}
}
