//! Conversion to a `JlrsResult`.

use crate::error::{exception, CallResult, JlrsResult};

/// Implementors of this trait can be converted to a [`JlrsResult`]. This trait is implemented
/// for [`CallResult`] to convert an exception thrown by Julia into an error message.
pub trait IntoJlrsResult<T> {
    /// Convert `self` to `JlrsResult`.
    fn into_jlrs_result(self) -> JlrsResult<T>;
}

impl<T> IntoJlrsResult<T> for CallResult<'_, '_, T> {
    fn into_jlrs_result(self) -> JlrsResult<T> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => exception(format!("{:?}", e)),
        }
    }
}
