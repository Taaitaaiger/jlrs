//! Convert a `JuliaResult` to a `JlrsResult`.
//!
//! A `JuliaResult` contains an exception in its `Err` variant, if you're only interested in
//! the error message you can convert it to a `JlrsException` with the [`IntoJlrsResult`] trait
//! defined in this module.

use crate::{
    error::{JlrsError, JlrsResult, JuliaResult, CANNOT_DISPLAY_VALUE},
    wrappers::ptr::Wrapper,
};

/// Extension trait that lets you convert a `JuliaResult` to a `JlrsResult`.
///
/// If an exception is thrown, this trait's only method converts the exception to an error message
/// by calling `Base.showerror`.
pub trait IntoJlrsResult<T>: private::IntoJlrsResultPriv {
    /// Convert `self` to `JlrsResult` by calling `Base.showerror` if an exception has been
    /// thrown.
    fn into_jlrs_result(self) -> JlrsResult<T>;
}

impl<T> IntoJlrsResult<T> for JuliaResult<'_, '_, T> {
    #[inline]
    fn into_jlrs_result(self) -> JlrsResult<T> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => JlrsError::exception_error(e.error_string_or(CANNOT_DISPLAY_VALUE))?,
        }
    }
}

mod private {
    use crate::error::JuliaResult;

    pub trait IntoJlrsResultPriv {}
    impl<T> IntoJlrsResultPriv for JuliaResult<'_, '_, T> {}
}
