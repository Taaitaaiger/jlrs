use std::mem::MaybeUninit;

use crate::data::managed::{Managed, Weak};

/// Convert `&mut T` to `&mut MaybeUninit<T>`.
pub unsafe trait AsMaybeUninit: Sized {
    fn as_maybe_uninit(&mut self) -> &mut MaybeUninit<Self>;
}

unsafe impl<'scope, 'data, T: Managed<'scope, 'data>> AsMaybeUninit
    for Option<Weak<'scope, 'data, T>>
{
    fn as_maybe_uninit(&mut self) -> &mut MaybeUninit<Self> {
        unsafe { &mut *(self as *mut _ as *mut MaybeUninit<Self>) }
    }
}
