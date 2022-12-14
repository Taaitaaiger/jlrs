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

use super::{private::ManagedPriv, Managed};
use crate::{
    data::{
        layout::foreign::{create_foreign_type, ForeignType},
        managed::{module::Module, symbol::Symbol, value::Value},
    },
    memory::target::{output::Output, RootingTarget},
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
pub struct WithParachute<'scope, T: Sync + 'static> {
    data: &'scope mut Option<T>,
}

impl<'scope, T: 'static + Sync> WithParachute<'scope, T> {
    /// Remove the parachute.
    ///
    /// Returns ownership of the data from Julia to Rust.
    pub fn remove_parachute(self) -> T {
        self.data.take().expect("Data is None")
    }
}

impl<'scope, T: 'static + Sync> Deref for WithParachute<'scope, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.data.as_ref().expect("Data is None")
    }
}

impl<'scope, T: 'static + Sync> DerefMut for WithParachute<'scope, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data.as_mut().expect("Data is None")
    }
}

/// Attach a parachute to this data to ensure it's safely dropped if Julia jumps.
pub trait AttachParachute: 'static + Sized + Sync {
    /// Attach a parachute to this data.
    ///
    /// By attaching a parachute, you move ownership of the data from Rust to Julia. This ensures
    /// the data is freed by Julia's GC after it has become unreachable.
    fn attach_parachute<'scope, T: RootingTarget<'scope>>(
        self,
        target: T,
    ) -> WithParachute<'scope, Self> {
        // Parachute::<Self>::register(frame)
        let mut output = target.into_output();
        Parachute::<Self>::register(&mut output);
        let parachute = Parachute { _data: Some(self) };
        let data = Value::new(output, parachute);
        unsafe {
            let mut ptr: NonNull<Option<Self>> = data.unwrap_non_null(Private).cast();
            WithParachute { data: ptr.as_mut() }
        }
    }
}

impl<T: 'static + Sized + Sync> AttachParachute for T {}

#[repr(transparent)]
pub(crate) struct Parachute<T: Sync + 'static> {
    _data: Option<T>,
}

// Safety: `T` contains no references to Julia data to the default implementation of `mark` is
// correct.
unsafe impl<T: Sync + 'static> ForeignType for Parachute<T> {}

struct UnitHasher(u64);

#[inline(always)]
fn to_8_u8s(data: &[u8]) -> [u8; 8] {
    let mut out = [0u8; 8];
    for i in 0..data.len().min(8) {
        out[i] = data[i];
    }
    out
}

impl Hasher for UnitHasher {
    #[inline(always)]
    fn finish(&self) -> u64 {
        self.0
    }

    #[inline(always)]
    fn write(&mut self, bytes: &[u8]) {
        self.0 = u64::from_ne_bytes(to_8_u8s(bytes));
    }
}

impl<T: Sync + 'static> Parachute<T> {
    fn register<'scope>(output: &mut Output<'scope>) {
        let type_id = TypeId::of::<T>();
        debug_assert_eq!(std::mem::size_of_val(&type_id), 8);

        let mut hasher = UnitHasher(0);
        type_id.hash(&mut hasher);
        let hashed = hasher.finish();
        let name = format!("__Parachute_{:x}__", hashed);
        let sym = Symbol::new(&output, name.as_str());
        let module = Module::main(&output);

        if module.global(&output, sym).is_ok() {
            return;
        }

        unsafe {
            let dt = create_foreign_type::<Self, _>(output, sym, module, None, false, false);
            module.set_const_unchecked(sym, dt.as_value());
        }
    }
}
