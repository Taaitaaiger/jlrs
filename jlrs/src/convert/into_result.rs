//! Convert data to a `Result`.

use crate::error::JlrsResult;

/// Convert `Self` to a `Result`.
pub trait IntoResult<T, E> {
    /// Convert `self` to a `Result`.
    fn into_result(self) -> Result<T, E>;
}

impl<E> IntoResult<(), E> for () {
    fn into_result(self) -> Result<(), E> {
        Ok(self)
    }
}

impl<E> IntoResult<JlrsResult<()>, E> for JlrsResult<()> {
    fn into_result(self) -> Result<JlrsResult<()>, E> {
        Ok(self)
    }
}

impl<E> IntoResult<(), E> for Result<(), E> {
    fn into_result(self) -> Result<(), E> {
        self
    }
}

impl<E> IntoResult<JlrsResult<()>, E> for Result<JlrsResult<()>, E> {
    fn into_result(self) -> Result<JlrsResult<()>, E> {
        self
    }
}
