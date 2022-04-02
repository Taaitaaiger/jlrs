//! Access token for global Julia data.
//!
//! Some data in Julia is globally rooted. This includes the `Main`, `Base` and `Core` modules,
//! symbols, and `DataType`s of builtin types. In order to access this data, jlrs requires a
//! [`Global`] in order to prevent it from being accessed before Julia has been initialized.
//!
//! Another use-case for [`Global`] is calling Julia functions without rooting the result. This is
//! useful if you don't need to use the result of a function call, if it returns `nothing` for
//! example.

use std::marker::PhantomData;

use crate::info::Info;

/// Access token required for accessing global Julia data, also used to call Julia function
/// without rooting the result.
#[derive(Copy, Clone)]
pub struct Global<'global>(PhantomData<&'global ()>);

impl<'global> Global<'global> {
    // Safety: Julia must have been initialized
    pub(crate) unsafe fn new() -> Self {
        Global(PhantomData)
    }

    /// Provides access to global information.
    pub fn info(self) -> Info {
        unsafe { Info::new() }
    }
}
