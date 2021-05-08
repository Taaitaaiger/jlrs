//! Convert a Julia value to Rust.
//!
//! The trait in this module should be implemented by deriving `JuliaStruct`, its methods are
//! never called directly but only through [`Value::cast`] and [`Value::cast_unchecked`].

use crate::error::JlrsResult;
use crate::value::Value;

/// This trait is implemented by types that a [`Value`] can be converted into by calling
/// [`Value::cast`]. This includes types like `String`, [`Array`], and `u8`.
///
/// [`Array`]: crate::value::array::Array
pub unsafe trait Cast<'frame, 'data> {
    type Output;
    #[doc(hidden)]
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output>;

    #[doc(hidden)]
    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output;
}
