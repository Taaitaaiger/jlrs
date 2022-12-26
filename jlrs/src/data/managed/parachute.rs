//! Managed type for some Rust data.
//!
//! Throwing an exception in Julia is implemented by jumping to the nearest enclosing catch block.
//! This can be problematic, particularly from `ccall`ed functions, because jumping over a Rust
//! function with pending drops can prevent that data from being dropped at best, and is terribly
//! UB at worst.
//!
//! In order to ensure data is safely dropped even if Julia jumps, you can attach a parachute by
//! calling [`AttachParachute::attach_parachute`] to transfer ownership of the data from Rust to
//! Julia. This method is available if `Self: 'static + Sized + Send `. The data must be `'static`
//! because there are no guarantees about drop order, `Sized` because ownership of the data is
//! moved to Julia, and `Sync` because the GC is allowed to drop the data from another thread.

use std::{
    any::TypeId,
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use super::private::ManagedPriv;
use crate::{
    data::{
        layout::foreign::{create_foreign_type_internal, ForeignType},
        managed::{datatype::DataType, module::Module, symbol::Symbol, value::Value},
    },
    memory::{
        stack_frame::StackFrame,
        target::{unrooted::Unrooted, RootingTarget},
    },
    private::Private,
};

/// Data that has been protected with a parachute.
///
/// When a parachute is attached to data with [`AttachParachute::attach_parachute`], ownership of
/// the data is transfered from Rust to Julia. This ensures the data will be dropped safely, even
/// if Julia throws an exception and would have jumped over the pending drop without a parachute.
///
/// Unlike other managed types, `WithParachute` doesn't implement [`Managed`] but behaves like a
/// mutable reference to the protected data: it implements `Deref` and `DerefMut` to allow using
/// protected data as if it were a mutable reference to the original data. The  parachute can be
/// removed by calling `WithParachute::remove_parachute` to regain ownership.
///
/// For more information, see the [module-level docs].
///
/// [`Managed`]: crate::data::managed::Managed
/// [module-level docs]: self
pub struct WithParachute<'scope, T> {
    data: &'scope mut Option<T>,
}

impl<'scope, T> WithParachute<'scope, T> {
    /// Remove the parachute.
    ///
    /// Returns ownership of the data from Julia to Rust.
    pub fn remove_parachute(self) -> T {
        self.data.take().expect("Data is None")
    }
}

impl<'scope, T> Deref for WithParachute<'scope, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.data.as_ref().expect("Data is None")
    }
}

impl<'scope, T> DerefMut for WithParachute<'scope, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data.as_mut().expect("Data is None")
    }
}

/// Attach a parachute to this data to ensure it's safely dropped if Julia jumps.
pub trait AttachParachute: 'static + Sized + Send + Sync {
    /// Attach a parachute to this data.
    ///
    /// By attaching a parachute, you move ownership of the data from Rust to Julia. This ensures
    /// the data is freed by Julia's GC after it has become unreachable.
    fn attach_parachute<'scope, T: RootingTarget<'scope>>(
        self,
        target: T,
    ) -> WithParachute<'scope, Self> {
        let output = target.into_output();
        let parachute = Parachute { _data: Some(self) };
        let data = Value::new(output, parachute);
        unsafe {
            let mut ptr: NonNull<Option<Self>> = data.unwrap_non_null(Private).cast();
            WithParachute { data: ptr.as_mut() }
        }
    }
}

impl<T: 'static + Sized + Send + Sync> AttachParachute for T {}

#[repr(transparent)]
pub(crate) struct Parachute<T: Sync + 'static> {
    _data: Option<T>,
}

// Safety: `T` contains no references to Julia data so the default implementation of `mark` is
// correct.
unsafe impl<T: Send + Sync + 'static> ForeignType for Parachute<T> {
    const TYPE_FN: Option<unsafe fn() -> DataType<'static>> = Some(init_foreign::<Self>);
}

#[doc(hidden)]
unsafe fn init_foreign<T: ForeignType>() -> DataType<'static> {
    let mut hasher = hashers::fnv::FNV1aHasher64::default();
    let type_id = TypeId::of::<T>();
    type_id.hash(&mut hasher);
    let type_id_hash = hasher.finish();

    let name = format!("__Parachute_{:x}__", type_id_hash);

    unsafe {
        let unrooted = Unrooted::new();
        let sym = Symbol::new(&unrooted, name.as_str());
        let module = Module::main(&unrooted);

        let mut frame = StackFrame::new();
        let pinned = frame.pin();
        let dt = create_foreign_type_internal::<T, _>(unrooted, sym, module, None, false, false);
        pinned.set_sync_root(dt.ptr().as_ptr().cast());
        module.set_const_unchecked(sym, dt.as_value());
        std::mem::drop(pinned);

        DataType::wrap_non_null(dt.ptr(), Private)
    }
}
