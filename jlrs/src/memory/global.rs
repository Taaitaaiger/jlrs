//! Access token for global Julia data.
//!
//! Some data in Julia is globally rooted. This includes the `Main`, `Base` and `Core` modules,
//! symbols, and `DataType`s of builtin types. In order to access this data, jlrs requires a
//! [`Global`] in order to prevent it from being accessed before Julia has been initialized.
//!
//! Another use-case for [`Global`] is calling Julia functions without rooting the result. This is
//! useful if you don't need to use the result of a function call, if it returns `nothing` for
//! example.

use std::{marker::PhantomData, cell::RefCell, ops::Deref};

use crate::info::Info;

use super::ledger::Ledger;

/// Access token required for accessing global Julia data, also used to call Julia function
/// without rooting the result.
#[derive(Copy, Clone)]
pub struct Global<'global>(PhantomData<&'global ()>);

#[derive(Copy, Clone)]
pub struct BetterGlobal<'global>(Global<'global>, &'global RefCell<Ledger>);

impl<'global> Deref for BetterGlobal<'global> {
    type Target = Global<'global>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'global> Global<'global> {
    // Safety: this function must only be called from a thread known to Julia.
    pub(crate) unsafe fn new() -> Self {
        Global(PhantomData)
    }

    /// Provides access to global information.
    pub fn info(self) -> Info {
        unsafe { Info::new() }
    }
}

impl<'global> BetterGlobal<'global> {
    // Safety: this function must only be called from a thread known to Julia.
    pub(crate) unsafe fn new(ledger: &'global RefCell<Ledger>) -> Self {
        BetterGlobal(Global(PhantomData), ledger)
    }

    /// Provides access to global information.
    pub fn info(self) -> Info {
        unsafe { Info::new() }
    }
}
