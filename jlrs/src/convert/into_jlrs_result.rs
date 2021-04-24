//! Convert a `JuliaResult` to a `JlrsResult`.

use crate::error::{exception, JlrsResult, JuliaResult};

/// Implementors of this trait can be converted to a [`JlrsResult`]. This trait is implemented
/// for [`JuliaResult`] to convert an exception thrown by Julia into an error message.
pub trait IntoJlrsResult<T> {
    /// Convert `self` to `JlrsResult`.
    fn into_jlrs_result(self) -> JlrsResult<T>;
}

impl<T> IntoJlrsResult<T> for JuliaResult<'_, '_, T> {
    fn into_jlrs_result(self) -> JlrsResult<T> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => exception(format!("{:?}", e)),
        }
    }
}
