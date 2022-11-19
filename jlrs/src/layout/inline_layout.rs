//! Marker trait automatically implemented by types that provide a matching layout for Julia data.
use super::valid_layout::ValidLayout;
use crate::convert::unbox::Unbox;

/// Marker trait automatically implemented by types that provide a matching layout for Julia data.
pub trait InlineLayout: ValidLayout + Unbox<Output = Self> {}

impl<T: ValidLayout + Unbox<Output = Self>> InlineLayout for T {}
