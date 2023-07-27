//! Convert data to a `Result`.

use crate::error::JlrsResult;

/// Convert `Self` to a `Result`.
pub trait IntoResult<T, E> {
    /// Convert `self` to a `Result`.
    fn into_result(self) -> Result<T, E>;
}

impl<E> IntoResult<(), E> for () {
    #[inline]
    fn into_result(self) -> Result<(), E> {
        Ok(self)
    }
}

impl<E> IntoResult<JlrsResult<()>, E> for JlrsResult<()> {
    #[inline]
    fn into_result(self) -> Result<JlrsResult<()>, E> {
        Ok(self)
    }
}

impl<E> IntoResult<(), E> for Result<(), E> {
    #[inline]
    fn into_result(self) -> Result<(), E> {
        self
    }
}

impl<E> IntoResult<JlrsResult<()>, E> for Result<JlrsResult<()>, E> {
    #[inline]
    fn into_result(self) -> Result<JlrsResult<()>, E> {
        self
    }
}
