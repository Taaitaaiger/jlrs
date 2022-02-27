//! Access token for global Julia data.
//!
//! Some data in Julia is globally rooted. This includes the `Main`, `Base` and `Core` modules,
//! symbols, and `DataType`s of builtin types. In order to access this data, jlrs requires a
//! [`Global`] in order to prevent it from being accessed before Julia has been initialized.
//!
//! Another use-case for [`Global`] is calling Julia functions without rooting the result.

use std::marker::PhantomData;

/// Access token required for accessing global Julia data, also used to call Julia function
/// without rooting the result.
#[derive(Copy, Clone)]
pub struct Global<'global>(PhantomData<&'global ()>);

impl<'global> Global<'global> {
    // Safety: Julia must have been initialized
    pub(crate) unsafe fn new() -> Self {
        Global(PhantomData)
    }
}
