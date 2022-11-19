//! A non-rooting target.
//!
//! While any target can be used as a non-rooting target by using a reference to that target, this
//! can be problematic in nested expressions.

use std::marker::PhantomData;

/// A non-rooting target.
///
/// A new [`Unrooted`] can be created with [`Target::unrooted`].
///
/// [`Target::unrooted`]: crate::memory::target::Target::unrooted
#[derive(Copy, Clone, Debug)]
pub struct Unrooted<'target> {
    _marker: PhantomData<&'target ()>,
}

impl<'target> Unrooted<'target> {
    pub(crate) unsafe fn new() -> Self {
        Unrooted {
            _marker: PhantomData,
        }
    }
}
