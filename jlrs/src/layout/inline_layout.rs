//! Marker trait automatically implemented by types that provide a matching layout for Julia data.
use crate::convert::unbox::Unbox;

use super::valid_layout::ValidLayout;

/// Marker trait automatically implemented by types that provide a matching layout for Julia data.
///
/// Safety: the layout of the data in Rust and Julia must match exactly.
pub unsafe trait InlineLayout: ValidLayout + Unbox<Output = Self> {}

unsafe impl<T: ValidLayout + Unbox<Output = Self>> InlineLayout for T {}
