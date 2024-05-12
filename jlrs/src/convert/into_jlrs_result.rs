//! Convert data to a `JlrsResult`.

use crate::{
    data::managed::Managed,
    error::{JlrsError, JlrsResult, JuliaResult, CANNOT_DISPLAY_VALUE},
};

/// Convert data to a `JlrsResult`.
///
/// By default this trait is only implemented for `JuliaResult`. If an exception is thrown, it's
/// converted to an error message by calling `Base.showerror`.
pub trait IntoJlrsResult<T> {
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
