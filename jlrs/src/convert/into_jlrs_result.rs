//! Convert a `JuliaResult` to a `JlrsResult`.
//!
//! A [`JuliaResult`] is an alias for `Result` that's used when a function can throw an exception
//! that can be caught. This is currently limited to calling Julia functions and evaluating raw
//! Julia code. Inside a scope, the `?` operator can only be used with another alias for `Result`,
//! [`JlrsError`]. The [`IntoJlrsResult`] trait defined in this module can be used to convert a
//! `JuliaResult` to a [`JlrsResult`] to convert the exception to a simple error message which can
//! be returned from the closure.
//!
//! [`JlrsError`]: crate::error::JlrsError

use crate::{
    error::{exception, JlrsResult, JuliaResult, JuliaResultRef, CANNOT_DISPLAY_VALUE},
    prelude::Wrapper,
};

/// Extension trait that lets you convert a `JuliaResult` to a `JlrsResult`.
pub trait IntoJlrsResult<T>: private::IntoJlrsResult {
    /// Convert `self` to `JlrsResult`.
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

impl<T> IntoJlrsResult<T> for JuliaResultRef<'_, '_, T> {
    fn into_jlrs_result(self) -> JlrsResult<T> {
        unsafe {
            match self {
                Ok(v) => Ok(v),
                Err(e) => exception(e.value_unchecked().error_string_or(CANNOT_DISPLAY_VALUE)),
            }
        }
    }
}

mod private {
    use crate::error::{JuliaResult, JuliaResultRef};

    pub trait IntoJlrsResult {}
    impl<T> IntoJlrsResult for JuliaResult<'_, '_, T> {}
    impl<T> IntoJlrsResult for JuliaResultRef<'_, '_, T> {}
}
