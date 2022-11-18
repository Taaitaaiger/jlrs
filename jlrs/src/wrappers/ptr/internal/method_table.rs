//! Wrapper for `MethodTable`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#L535

use crate::{
    impl_julia_typecheck,
    prelude::{Module, Symbol, Target},
    private::Private,
    wrappers::ptr::{
        array::{ArrayData, ArrayRef},
        private::WrapperPriv,
        value::{ValueData, ValueRef},
        Ref,
    },
};
use cfg_if::cfg_if;
use jl_sys::{jl_methtable_t, jl_methtable_type};
use std::{marker::PhantomData, ptr::NonNull};

cfg_if! {
    if #[cfg(not(feature = "lts"))] {
        use std::sync::atomic::Ordering;
    }
}

/// contains the TypeMap for one Type
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct MethodTable<'scope>(NonNull<jl_methtable_t>, PhantomData<&'scope ()>);

impl<'scope> MethodTable<'scope> {
    /*
    inspect(Core.MethodTable):

    name: Symbol (mut)
    defs: Any (mut) _Atomic
    leafcache: Any (mut) _Atomic
    cache: Any (mut) _Atomic
    max_args: Int64 (mut)
    module: Module (const)
    backedges: Vector{Any} (mut)
    : Int64 (mut)
    : Int64 (mut)
    offs: UInt8 (mut)
    : UInt8 (mut)
    */

    /// Sometimes a hack used by serialization to handle kwsorter
    pub fn name(self) -> Option<Symbol<'scope>> {
        // Safety: the pointer points to valid data
        unsafe {
            let name = self.unwrap_non_null(Private).as_ref().name;
            let name = NonNull::new(name)?;
            Some(Symbol::wrap_non_null(name, Private))
        }
    }

    /// The `defs` field.
    pub fn defs<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe {
                    let data = self.unwrap_non_null(Private).as_ref().defs;
                    let data = NonNull::new(data)?;
                    Some(ValueRef::wrap(data).root(target))
                }
            } else {
                // Safety: the pointer points to valid data
                unsafe {
                    let data = self.unwrap_non_null(Private).as_ref().defs.load(Ordering::Relaxed);
                    let data = NonNull::new(data)?;
                    Some(ValueRef::wrap(data).root(target))
                }
            }
        }
    }

    /// The `leafcache` field.
    pub fn leafcache<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe {
                    let data = self.unwrap_non_null(Private).as_ref().leafcache;
                    let data = NonNull::new(data)?;
                    Some(ValueRef::wrap(data.cast()).root(target))
                }
            } else {
                // Safety: the pointer points to valid data
                unsafe {
                    let data = self.unwrap_non_null(Private).as_ref().leafcache.load(Ordering::Relaxed);
                    let data = NonNull::new(data)?;
                    Some(ValueRef::wrap(data.cast()).root(target))
                }
            }
        }
    }

    /// The `cache` field.
    pub fn cache<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe {
                    let data = self.unwrap_non_null(Private).as_ref().cache;
                    let data = NonNull::new(data)?;
                    Some(ValueRef::wrap(data).root(target))
                }
            } else {
                // Safety: the pointer points to valid data
                unsafe {
                    let data = self.unwrap_non_null(Private).as_ref().cache.load(Ordering::Relaxed);
                    let data = NonNull::new(data)?;
                    Some(ValueRef::wrap(data).root(target))
                }
            }
        }
    }

    /// Max # of non-vararg arguments in a signature
    pub fn max_args(self) -> isize {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().max_args }
    }

    #[cfg(not(any(feature = "nightly", feature = "beta")))]
    /// Keyword argument sorter function
    pub fn kw_sorter<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let kw_sorter = self.unwrap_non_null(Private).as_ref().kwsorter;
            let kw_sorter = NonNull::new(kw_sorter)?;
            Some(ValueRef::wrap(kw_sorter).root(target))
        }
    }

    /// Used for incremental serialization to locate original binding
    pub fn module(self) -> Option<Module<'scope>> {
        // Safety: the pointer points to valid data
        unsafe {
            let module = self.unwrap_non_null(Private).as_ref().module;
            let module = NonNull::new(module)?;
            Some(Module::wrap_non_null(module, Private))
        }
    }

    /// The `backedges` field.
    pub fn backedges<'target, T>(self, target: T) -> Option<ArrayData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let backedges = self.unwrap_non_null(Private).as_ref().backedges;
            let backedges = NonNull::new(backedges)?;
            Some(ArrayRef::wrap(backedges).root(target))
        }
    }

    /// 0, or 1 to skip splitting typemap on first (function) argument
    pub fn offs(self) -> u8 {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().offs }
    }

    /// Whether this accepts adding new methods
    pub fn frozen(self) -> u8 {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().frozen }
    }
}

impl_julia_typecheck!(MethodTable<'scope>, jl_methtable_type, 'scope);
impl_debug!(MethodTable<'_>);

impl<'scope> WrapperPriv<'scope, '_> for MethodTable<'scope> {
    type Wraps = jl_methtable_t;
    type TypeConstructorPriv<'target, 'da> = MethodTable<'target>;
    const NAME: &'static str = "<MethodTable";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

/// A reference to a [`MethodTable`] that has not been explicitly rooted.
pub type MethodTableRef<'scope> = Ref<'scope, 'static, MethodTable<'scope>>;
impl_valid_layout!(MethodTableRef, MethodTable);
impl_ref_root!(MethodTable, MethodTableRef, 1);

use crate::memory::target::target_type::TargetType;

/// `MethodTable` or `MethodTableRef`, depending on the target type `T`.
pub type MethodTableData<'target, T> =
    <T as TargetType<'target>>::Data<'static, MethodTable<'target>>;

/// `JuliaResult<MethodTable>` or `JuliaResultRef<MethodTableRef>`, depending on the target type
/// `T`.
pub type MethodTableResult<'target, T> =
    <T as TargetType<'target>>::Result<'static, MethodTable<'target>>;
