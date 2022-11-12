//! Marker trait automatically implemented by types that provide a matching layout for Julia data.
use crate::convert::unbox::Unbox;

use super::valid_layout::{ValidField, ValidLayout};

/// Marker trait automatically implemented by types that provide a matching layout for Julia data.
///
/// Safety: the layout of the data in Rust and Julia must match exactly.
pub unsafe trait InlineLayout: ValidLayout + ValidField + Unbox<Output = Self> {}

unsafe impl<T: ValidLayout + ValidField + Unbox<Output = Self>> InlineLayout for T {}
