//! Marker trait for layouts that only use `isbits` types.
//!
//! An `isbits` type is an immutable type that doesn't contain any references to Julia data.

/// Indicates that this type is an `isbits` type.
///
/// Safety: `Self` must map to an `isbits`-type
pub unsafe trait IsBits: 'static {}

unsafe impl IsBits for bool {}
unsafe impl IsBits for i8 {}
unsafe impl IsBits for i16 {}
unsafe impl IsBits for i32 {}
unsafe impl IsBits for i64 {}
unsafe impl IsBits for isize {}
unsafe impl IsBits for u8 {}
unsafe impl IsBits for u16 {}
unsafe impl IsBits for u32 {}
unsafe impl IsBits for u64 {}
unsafe impl IsBits for usize {}
unsafe impl IsBits for f32 {}
unsafe impl IsBits for f64 {}
unsafe impl<T: 'static> IsBits for *mut T {}
