//! A non-rooting target.
//!
//! While any target can be used as a non-rooting target by using a reference to that target,
//! targets are aware of their return type and can't be used as a non-rooting target with methods
//! that return data of a different type. This can be particularly problematic with methods that
//! take a generic target.

use std::marker::PhantomData;

/// A non-rooting target.
///
/// A new [`Unrooted`] can be created with [`Target::unrooted`].
///
/// [`Target::unrooted`]: crate::memory::target::Target::unrooted
#[derive(Copy, Clone, Debug)]
pub struct Unrooted<'scope> {
    _marker: PhantomData<&'scope ()>,
}

impl<'scope> Unrooted<'scope> {
    pub(crate) unsafe fn new() -> Self {
        Unrooted {
            _marker: PhantomData,
        }
    }
}
