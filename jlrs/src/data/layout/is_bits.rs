//! Marker trait for layouts that only use `isbits` types.
//!
//! An `isbits` type is an immutable type that doesn't contain any references to Julia data. The
//! layout of such a type is compatible with C, which means that the low-level layout of such a
//! type is just a `repr(C)` struct in Rust which contains the fields in the same order as the
//! Julia type, where all primitive types have been replaced with their Rusty counterparts.
//!
//! When every field of a layout is an `isbits` type we can assume that they contain valid data
//! and avoid performing any validity checks when converting it from Rust to Julia data. Types can
//! implement [`IsBits`] to indicate this.

use std::ffi::c_void;

use super::valid_layout::ValidLayout;

/// Indicate that all fields are `isbits` types.
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
unsafe impl IsBits for *mut c_void {}
unsafe impl<T: ValidLayout + 'static> IsBits for *mut T {}
