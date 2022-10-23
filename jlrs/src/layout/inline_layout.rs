// TODO Docs
use crate::convert::unbox::Unbox;

use super::valid_layout::ValidLayout;

pub unsafe trait InlineLayout: ValidLayout + Unbox<Output = Self> {}

unsafe impl<T: ValidLayout + Unbox<Output = Self>> InlineLayout for T {}
