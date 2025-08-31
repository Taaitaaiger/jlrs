use std::{fmt, marker::PhantomData};

use super::IsActive;
use crate::memory::scope::{LocalScope, private::LocalScopePriv};

/// A handle that lets you call into Julia from a delegated task.
pub struct DelegatedHandle {
    _marker: PhantomData<*mut ()>,
}

impl DelegatedHandle {
    pub(crate) unsafe fn new() -> Self {
        DelegatedHandle {
            _marker: PhantomData,
        }
    }
}

impl fmt::Debug for DelegatedHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DelegatedHandle").finish()
    }
}

impl IsActive for DelegatedHandle {}

impl LocalScopePriv for DelegatedHandle {}
unsafe impl LocalScope for DelegatedHandle {}
