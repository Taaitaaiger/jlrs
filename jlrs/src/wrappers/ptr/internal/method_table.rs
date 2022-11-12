//! Wrapper for `MethodTable`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#L535

use crate::{
    impl_julia_typecheck,
    prelude::Symbol,
    private::Private,
    wrappers::ptr::{
        array::ArrayRef, module::ModuleRef, private::WrapperPriv, value::ValueRef, Ref,
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
    for (a, b) in zip(fieldnames(Core.MethodTable), fieldtypes(Core.MethodTable))
        println(a, ": ", b)
    end
    name: Symbol
    defs: Any _Atomic
    leafcache: Any _Atomic
    cache: Any _Atomic
    max_args: Int64
    kwsorter: Any
    module: Module
    backedges: Vector{Any}
    : Int64
    : Int64
    offs: UInt8
    : UInt8
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
    pub fn defs(self) -> Option<ValueRef<'scope, 'static>> {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe {
                    let data = self.unwrap_non_null(Private).as_ref().defs;
                    let data = NonNull::new(data)?;
                    ValueRef::wrap(data)
                }
            } else {
                // Safety: the pointer points to valid data
                unsafe {
                    let data = self.unwrap_non_null(Private).as_ref().defs.load(Ordering::Relaxed);
                    let data = NonNull::new(data)?;
                    Some(ValueRef::wrap(data))
                }
            }
        }
    }

    /// The `leafcache` field.
    pub fn leafcache(self) -> Option<ArrayRef<'scope, 'static>> {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe {
                    let data = self.unwrap_non_null(Private).as_ref().leafcache;
                    let data = NonNull::new(data)?;
                    ArrayRef::wrap(data)
                }
            } else {
                // Safety: the pointer points to valid data
                unsafe {
                    let data = self.unwrap_non_null(Private).as_ref().leafcache.load(Ordering::Relaxed);
                    let data = NonNull::new(data)?;
                    Some(ArrayRef::wrap(data))
                }
            }
        }
    }

    /// The `cache` field.
    pub fn cache(self) -> Option<ValueRef<'scope, 'static>> {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe {
                    let data = self.unwrap_non_null(Private).as_ref().cache;
                    let data = NonNull::new(data)?;
                    ValueRef::wrap(data)
                }
            } else {
                // Safety: the pointer points to valid data
                unsafe {
                    let data = self.unwrap_non_null(Private).as_ref().cache.load(Ordering::Relaxed);
                    let data = NonNull::new(data)?;
                    Some(ValueRef::wrap(data))
                }
            }
        }
    }

    /// Max # of non-vararg arguments in a signature
    pub fn max_args(self) -> isize {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().max_args }
    }

    #[cfg(not(feature = "nightly"))]
    /// Keyword argument sorter function
    pub fn kw_sorter(self) -> ValueRef<'scope, 'static> {
        // Safety: the pointer points to valid data
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().kwsorter) }
    }

    /// Used for incremental serialization to locate original binding
    pub fn module(self) -> Option<ModuleRef<'scope>> {
        // Safety: the pointer points to valid data
        unsafe {
            let module = self.unwrap_non_null(Private).as_ref().module;
            let module = NonNull::new(module)?;
            Some(ModuleRef::wrap(module))
        }
    }

    /// The `backedges` field.
    pub fn backedges(self) -> Option<ArrayRef<'scope, 'static>> {
        // Safety: the pointer points to valid data
        unsafe {
            let backedges = self.unwrap_non_null(Private).as_ref().backedges;
            let backedges = NonNull::new(backedges)?;
            Some(ArrayRef::wrap(backedges))
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
