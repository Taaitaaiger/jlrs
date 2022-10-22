use jl_sys::jl_gc_wb;

use crate::{
    convert::unbox::Unbox, prelude::Value, private::Private, wrappers::ptr::private::WrapperPriv,
};

use super::valid_layout::ValidLayout;

pub unsafe trait InlineLayout: ValidLayout + Unbox {
    /// Updates the write barrier.
    ///
    /// When a pointer field of `self` has been set to `child`, this method must be called
    /// immediately after changing the field.
    ///
    /// Safety: TODO
    unsafe fn write_barrier(&mut self, child: Value) {
        jl_gc_wb(self as *mut _ as *mut _, child.unwrap(Private))
    }
}

unsafe impl<T: ValidLayout + Unbox> InlineLayout for T {}
