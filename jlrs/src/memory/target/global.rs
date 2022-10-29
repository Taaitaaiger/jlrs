//! A non-rooting target.
//!
//! While any target can be used as a non-rooting target by using a reference to that target,
//! targets are aware of their return type and can't be used as a non-rooting target with methods
//! that return data of a different type. This can be particularly problematic with methods that
//! take a generic target.

use std::marker::PhantomData;

/// A non-rooting target.
///
/// A new [`Global`] can be created with [`Target::global`].
///
/// [`Target::global`]: crate::memory::target::Target::global
#[derive(Copy, Clone, Debug)]
pub struct Global<'scope> {
    _marker: PhantomData<&'scope ()>,
}

impl<'scope> Global<'scope> {
    pub(crate) unsafe fn new() -> Self {
        Global {
            _marker: PhantomData,
        }
    }
}
