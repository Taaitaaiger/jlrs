//! Access token for global Julia data.

use std::marker::PhantomData;

/// Some kinds of values don't need to be protected from garbage collection, including
/// [`Symbol`]s, [`Module`]s, and functions and other globals defined in those modules. You will
/// need this struct to access these values, you acquire it when you create a base frame through
/// [`Julia::scope`] or [`Julia::scope_with_slots`].
///
/// [`Symbol`]: ../value/symbol/struct.Symbol.html
/// [`Module`]: ../value/module/struct.Module.html
/// [`Julia::scope_with_slots`]: crate::Julia::scope_with_slots
/// [`Julia::scope`]: crate::Julia::scope
#[derive(Copy, Clone)]
pub struct Global<'base>(PhantomData<&'base ()>);

impl<'base> Global<'base> {
    #[doc(hidden)]

    pub unsafe fn new() -> Self {
        Global(PhantomData)
    }
}
