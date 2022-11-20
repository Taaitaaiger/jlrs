//! Wrapper for `TypeMapEntry`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#505

use std::{marker::PhantomData, ptr::NonNull};

use cfg_if::cfg_if;
use jl_sys::{jl_typemap_entry_t, jl_typemap_entry_type};

use crate::{
    impl_julia_typecheck,
    memory::target::Target,
    private::Private,
    wrappers::ptr::{
        private::WrapperPriv,
        value::{Value, ValueData, ValueRef},
        Ref,
    },
};

cfg_if! {
    if #[cfg(not(feature = "lts"))] {
        use std::sync::atomic::Ordering;
    }
}

/// One Type-to-Value entry
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct TypeMapEntry<'scope>(NonNull<jl_typemap_entry_t>, PhantomData<&'scope ()>);

impl<'scope> TypeMapEntry<'scope> {
    /*
    inspect(Core.TypeMapEntry):

    next: Any (mut) _Atomic
    sig: Type (const)
    simplesig: Any (const)
    guardsigs: Any (const)
    min_world: UInt64 (const)
    max_world: UInt64 (const)
    func: Any (const)
    isleafsig: Bool (const)
    issimplesig: Bool (const)
    va: Bool (const)
    */

    /// Invasive linked list
    pub fn next<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe {
                    let next = self.unwrap_non_null(Private).as_ref().next.cast();
                    let next = NonNull::new(next)?;
                    Some(ValueRef::wrap(next).root(target))
                }
            } else {
                // Safety: the pointer points to valid data
                unsafe {
                    let next = self.unwrap_non_null(Private).as_ref().next.load(Ordering::Relaxed).cast();
                    let next = NonNull::new(next)?;
                    Some(ValueRef::wrap(next).root(target))
                }
            }
        }
    }

    /// The type sig for this entry
    pub fn sig(self) -> Option<Value<'scope, 'static>> {
        // Safety: the pointer points to valid data
        unsafe {
            let data = self.unwrap_non_null(Private).as_ref().sig.cast();
            let data = NonNull::new(data)?;
            Some(Value::wrap_non_null(data, Private))
        }
    }

    /// A simple signature for fast rejection
    pub fn simple_sig(self) -> Option<Value<'scope, 'static>> {
        // Safety: the pointer points to valid data
        unsafe {
            let data = self.unwrap_non_null(Private).as_ref().simplesig.cast();
            let data = NonNull::new(data)?;
            Some(Value::wrap_non_null(data, Private))
        }
    }

    /// The `guardsigs` field.
    pub fn guard_sigs(self) -> Option<Value<'scope, 'static>> {
        // Safety: the pointer points to valid data
        unsafe {
            let data = self.unwrap_non_null(Private).as_ref().guardsigs.cast();
            let data = NonNull::new(data)?;
            Some(Value::wrap_non_null(data, Private))
        }
    }

    /// The `min_world` field.
    pub fn min_world(self) -> usize {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().min_world }
    }

    /// The `max_world` field.
    pub fn max_world(self) -> usize {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().max_world }
    }

    /// The `func` field.
    pub fn func(self) -> Option<Value<'scope, 'static>> {
        // Safety: the pointer points to valid data
        unsafe {
            let data = self.unwrap_non_null(Private).as_ref().func.value;
            let data = NonNull::new(data)?;
            Some(Value::wrap_non_null(data, Private))
        }
    }

    /// `isleaftype(sig) & !any(isType, sig)` : unsorted and very fast
    pub fn is_leaf_signature(self) -> bool {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().isleafsig != 0 }
    }

    /// `all(isleaftype | isAny | isType | isVararg, sig)` : sorted and fast
    pub fn is_simple_signature(self) -> bool {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().issimplesig != 0 }
    }

    /// `isVararg(sig)`
    pub fn is_vararg(self) -> bool {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().va != 0 }
    }
}

impl_julia_typecheck!(TypeMapEntry<'scope>, jl_typemap_entry_type, 'scope);
impl_debug!(TypeMapEntry<'_>);

impl<'scope> WrapperPriv<'scope, '_> for TypeMapEntry<'scope> {
    type Wraps = jl_typemap_entry_t;
    type TypeConstructorPriv<'target, 'da> = TypeMapEntry<'target>;
    const NAME: &'static str = "TypeMapEntry";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

/// A reference to a [`TypeMapEntry`] that has not been explicitly rooted.
pub type TypeMapEntryRef<'scope> = Ref<'scope, 'static, TypeMapEntry<'scope>>;
impl_valid_layout!(TypeMapEntryRef, TypeMapEntry);

use crate::memory::target::target_type::TargetType;

/// `TypeMapEntry` or `TypeMapEntryRef`, depending on the target type `T`.
pub type TypeMapEntryData<'target, T> =
    <T as TargetType<'target>>::Data<'static, TypeMapEntry<'target>>;

/// `JuliaResult<TypeMapEntry>` or `JuliaResultRef<TypeMapEntryRef>`, depending on the target type
/// `T`.
pub type TypeMapEntryResult<'target, T> =
    <T as TargetType<'target>>::Result<'static, TypeMapEntry<'target>>;
