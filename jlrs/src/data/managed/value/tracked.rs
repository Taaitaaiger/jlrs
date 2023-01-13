//! Tracked Julia data.
//!
//! By tracking Julia data it's possible to ensure no aliasing rules are broken from Rust when
//! accessing their contents. While the data is tracked its contents can be derefenced.

use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::{
    data::{layout::valid_layout::ValidLayout, managed::value::Value},
    memory::context::ledger::Ledger,
};

/// Immutable tracked data.
#[repr(transparent)]
pub struct Tracked<'borrow, 'scope, 'data, T> {
    tracked: &'borrow T,
    _s: PhantomData<&'scope ()>,
    _d: PhantomData<&'data ()>,
}

impl<'borrow, 'scope, 'data, T: ValidLayout> Tracked<'borrow, 'scope, 'data, T> {
    pub(crate) unsafe fn new(value: &'borrow Value<'scope, 'data>) -> Self {
        Tracked {
            tracked: value.data_ptr().cast::<T>().as_ref(),
            _s: PhantomData,
            _d: PhantomData,
        }
    }
}

/// Mutable tracked data.
#[repr(transparent)]
pub struct TrackedMut<'borrow, 'scope, 'data, T: ValidLayout> {
    t: &'borrow mut T,
    _s: PhantomData<&'scope ()>,
    _d: PhantomData<&'data ()>,
}

impl<'borrow, 'scope, 'data, T: ValidLayout> TrackedMut<'borrow, 'scope, 'data, T> {
    pub(crate) unsafe fn new(value: &'borrow mut Value<'scope, 'data>) -> Self {
        TrackedMut {
            t: value.data_ptr().cast::<T>().as_mut(),
            _s: PhantomData,
            _d: PhantomData,
        }
    }
}

impl<'borrow, 'scope, 'data, T: ValidLayout> Deref for Tracked<'borrow, 'scope, 'data, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.tracked
    }
}

impl<'borrow, 'scope, 'data, T: ValidLayout> Deref for TrackedMut<'borrow, 'scope, 'data, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.t
    }
}

impl<'borrow, 'scope, 'data, T: ValidLayout> DerefMut for TrackedMut<'borrow, 'scope, 'data, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.t
    }
}

impl<T> Drop for Tracked<'_, '_, '_, T> {
    fn drop(&mut self) {
        unsafe {
            let start = self.tracked as *const _ as *mut u8;
            let end = start.add(std::mem::size_of::<T>());
            Ledger::unborrow_shared(start..end)
        }
    }
}

impl<T: ValidLayout> Drop for TrackedMut<'_, '_, '_, T> {
    fn drop(&mut self) {
        unsafe {
            let start = self.t as *const _ as *mut u8;
            let end = start.add(std::mem::size_of::<T>());
            Ledger::unborrow_owned(start..end)
        }
    }
}
