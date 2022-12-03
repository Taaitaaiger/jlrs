use std::ptr::NonNull;

use jl_sys::jl_value_t;

use super::Value;
use crate::{data::managed::private::ManagedPriv, memory::target::Target, private::Private};

/// While jlrs generally enforces that Julia data can only exist and be used while a frame is
/// active, it's possible to leak global values: [`Symbol`]s, [`Module`]s, and globals defined in
/// those modules.
#[derive(Copy, Clone)]
pub struct LeakedValue(Value<'static, 'static>);

impl LeakedValue {
    // Safety: ptr must point to valid Julia data
    pub(crate) unsafe fn wrap_non_null(ptr: NonNull<jl_value_t>) -> Self {
        LeakedValue(Value::wrap_non_null(ptr, Private))
    }

    /// Convert this [`LeakedValue`] back to a [`Value`]. This requires an [`Unrooted`], so this
    /// method can only be called inside a closure taken by one of the `scope`-methods.
    ///
    /// Safety: you must guarantee this value has not been freed by the garbage collector. While
    /// `Symbol`s are never garbage collected, modules and their contents can be redefined.
    #[inline(always)]
    pub unsafe fn as_value<'scope, T: Target<'scope>>(self, _: &T) -> Value<'scope, 'static> {
        self.0
    }
}
