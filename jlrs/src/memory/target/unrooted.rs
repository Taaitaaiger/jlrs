//! A weak target.
//!
//! While any target can be used as a weak target by using a reference to that target, this
//! can be problematic in nested expressions.

use std::marker::PhantomData;

/// A weak target.
///
/// A new [`Unrooted`] can be created with [`Target::unrooted`].
///
/// [`Target::unrooted`]: crate::memory::target::Target::unrooted
#[derive(Copy, Clone, Debug)]
pub struct Unrooted<'target> {
    _marker: PhantomData<&'target ()>,
}

impl<'target> Unrooted<'target> {
    #[inline]
    pub(crate) const unsafe fn new() -> Self {
        Unrooted {
            _marker: PhantomData,
        }
    }
}
