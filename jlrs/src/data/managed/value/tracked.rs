//! Tracked Julia data.
//!
//! Tracking data is the only way to get a (mutable) reference to Julia data in jlrs. Data can be
//! tracked mutably or immutably with [`Value::track_shared`] and [`Value::track_exclusive`].

use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use jl_sys::jl_value_t;

use crate::{
    data::{
        layout::valid_layout::ValidLayout,
        managed::{private::ManagedPriv, value::Value},
    },
    memory::context::ledger::Ledger,
    private::Private,
};

// TODO: Clone
/// Immutable tracked data.
#[repr(transparent)]
pub struct Tracked<'tracked, 'scope, 'data, T> {
    tracked: &'tracked T,
    _s: PhantomData<&'scope ()>,
    _d: PhantomData<&'data ()>,
}

impl<'tracked, 'scope, 'data, T: ValidLayout> Tracked<'tracked, 'scope, 'data, T> {
    #[inline]
    pub(crate) unsafe fn new(value: &'tracked Value<'scope, 'data>) -> Self {
        Tracked {
            tracked: value.data_ptr().cast::<T>().as_ref(),
            _s: PhantomData,
            _d: PhantomData,
        }
    }
}

impl<'scope, 'data, T: ValidLayout> Tracked<'scope, 'scope, 'data, T> {
    #[inline]
    pub(crate) unsafe fn new_owned(value: Value<'scope, 'data>) -> Self {
        Tracked {
            tracked: value.data_ptr().cast::<T>().as_ref(),
            _s: PhantomData,
            _d: PhantomData,
        }
    }
}

impl<'tracked, 'scope, 'data, T: ValidLayout> Deref for Tracked<'tracked, 'scope, 'data, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.tracked
    }
}

unsafe impl<'tracked, 'scope, 'data, T: ValidLayout + Send> Send
    for Tracked<'tracked, 'scope, 'data, T>
{
}

impl<T> Drop for Tracked<'_, '_, '_, T> {
    fn drop(&mut self) {
        unsafe {
            let v = Value::wrap_non_null(
                NonNull::new_unchecked(self.tracked as *const _ as *mut jl_value_t),
                Private,
            );

            if v.datatype().mutable() {
                Ledger::unborrow_shared(v).unwrap();
            }
        }
    }
}

/// Mutable tracked data.
#[repr(transparent)]
pub struct TrackedMut<'tracked, 'scope, 'data, T: ValidLayout> {
    t: &'tracked mut T,
    _s: PhantomData<&'scope ()>,
    _d: PhantomData<&'data ()>,
}

impl<'tracked, 'scope, 'data, T: ValidLayout> TrackedMut<'tracked, 'scope, 'data, T> {
    #[inline]
    pub(crate) unsafe fn new(value: &'tracked mut Value<'scope, 'data>) -> Self {
        TrackedMut {
            t: value.data_ptr().cast::<T>().as_mut(),
            _s: PhantomData,
            _d: PhantomData,
        }
    }
}

impl<'scope, 'data, T: ValidLayout> TrackedMut<'scope, 'scope, 'data, T> {
    #[inline]
    pub(crate) unsafe fn new_owned(value: Value<'scope, 'data>) -> Self {
        TrackedMut {
            t: value.data_ptr().cast::<T>().as_mut(),
            _s: PhantomData,
            _d: PhantomData,
        }
    }
}

impl<'tracked, 'scope, 'data, T: ValidLayout> Deref for TrackedMut<'tracked, 'scope, 'data, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.t
    }
}

impl<'tracked, 'scope, 'data, T: ValidLayout> DerefMut for TrackedMut<'tracked, 'scope, 'data, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.t
    }
}

unsafe impl<'tracked, 'scope, 'data, T: ValidLayout + Send> Send
    for TrackedMut<'tracked, 'scope, 'data, T>
{
}

impl<T: ValidLayout> Drop for TrackedMut<'_, '_, '_, T> {
    fn drop(&mut self) {
        unsafe {
            let v = Value::wrap_non_null(
                NonNull::new_unchecked(self.t as *mut _ as *mut jl_value_t),
                Private,
            );
            Ledger::unborrow_exclusive(v).unwrap();
        }
    }
}
