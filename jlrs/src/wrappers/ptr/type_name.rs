//! Wrapper for `TypeName`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#L380

use crate::{
    impl_julia_typecheck,
    memory::target::Target,
    private::Private,
    wrappers::ptr::{
        module::ModuleRef, private::WrapperPriv, simple_vector::SimpleVectorRef, symbol::SymbolRef,
    },
};
use cfg_if::cfg_if;
use jl_sys::{
    jl_array_typename, jl_llvmpointer_typename, jl_namedtuple_typename, jl_pointer_typename,
    jl_tuple_typename, jl_type_typename, jl_typename_t, jl_typename_type, jl_vecelement_typename,
};
use std::{marker::PhantomData, ptr::NonNull};

use super::Ref;

cfg_if! {
    if #[cfg(feature = "lts")] {
        use jl_sys::jl_vararg_typename;

    } else {
        use jl_sys::{jl_opaque_closure_typename};
    }
}

cfg_if! {
    if #[cfg(feature = "extra-fields")] {
        use crate::wrappers::ptr::value::ValueRef;
    }
}

cfg_if! {
    if #[cfg(all(not(feature = "lts"), feature = "extra-fields"))] {
        use std::sync::atomic::Ordering;
    }
}

/// Describes the syntactic structure of a type and stores all data common to different
/// instantiations of the type.
#[derive(Copy, Clone, PartialEq)]
#[repr(transparent)]
pub struct TypeName<'scope>(NonNull<jl_typename_t>, PhantomData<&'scope ()>);

impl<'scope> TypeName<'scope> {
    /*
    for (a, b) in zip(fieldnames(Core.TypeName), fieldtypes(Core.TypeName))
        println(a, ": ", b)
    end
    name: Symbol
    module: Module
    names: Core.SimpleVector
    atomicfields: Ptr{Nothing}
    wrapper: Type
    cache: Core.SimpleVector _Atomic
    linearcache: Core.SimpleVector _Atomic
    mt: Core.MethodTable
    partial: Any
    hash: Int64
    n_uninitialized: Int32
    flags: UInt8
    */

    /// The `name` field.
    pub fn name(self) -> SymbolRef<'scope> {
        // Safety: the pointer points to valid data
        unsafe {
            let name = self.unwrap_non_null(Private).as_ref().name;
            debug_assert!(!name.is_null());
            SymbolRef::wrap(NonNull::new_unchecked(name))
        }
    }

    /// The `module` field.
    pub fn module(self) -> ModuleRef<'scope> {
        // Safety: the pointer points to valid data
        unsafe {
            let module = self.unwrap_non_null(Private).as_ref().module;
            debug_assert!(!module.is_null());
            ModuleRef::wrap(NonNull::new_unchecked(module))
        }
    }

    /// Field names.
    pub fn names(self) -> Option<SimpleVectorRef<'scope>> {
        // Safety: the pointer points to valid data
        unsafe {
            let names = self.unwrap_non_null(Private).as_ref().names;
            let names = NonNull::new(names)?;
            Some(SimpleVectorRef::wrap(names))
        }
    }

    /// The `atomicfields` field.
    #[cfg(not(feature = "lts"))]
    pub fn atomicfields(self) -> *const u32 {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().atomicfields }
    }

    /// The `atomicfields` field.
    #[cfg(not(feature = "lts"))]
    pub fn constfields(self) -> *const u32 {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().constfields }
    }

    /// Either the only instantiation of the type (if no parameters) or a `UnionAll` accepting
    /// parameters to make an instantiation.
    #[cfg(feature = "extra-fields")]
    pub fn wrapper(self) -> Option<ValueRef<'scope, 'static>> {
        // Safety: the pointer points to valid data
        unsafe {
            let wrapper = self.unwrap_non_null(Private).as_ref().wrapper;
            let wrapper = NonNull::new(wrapper)?;
            Some(ValueRef::wrap(wrapper))
        }
    }

    /// Sorted array.
    #[cfg(feature = "extra-fields")]
    pub fn cache(self) -> SimpleVectorRef<'scope> {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe {
                    let cache = self.unwrap_non_null(Private).as_ref().cache;
                    debug_assert!(!cache.is_null());
                    SimpleVectorRef::wrap(NonNull::new_unchecked(cache))
                }
            } else {
                // Safety: the pointer points to valid data
                unsafe {
                    let cache = self.unwrap_non_null(Private).as_ref().cache.load(Ordering::Relaxed);
                    debug_assert!(!cache.is_null());
                    SimpleVectorRef::wrap(NonNull::new_unchecked(cache))
                }
            }
        }
    }

    /// Unsorted array.
    #[cfg(feature = "extra-fields")]
    pub fn linear_cache(self) -> SimpleVectorRef<'scope> {
        cfg_if! {
            if #[cfg(feature = "lts")] {
                // Safety: the pointer points to valid data
                unsafe {
                    let cache = self.unwrap_non_null(Private).as_ref().linearcache;
                    debug_assert!(!cache.is_null());
                    SimpleVectorRef::wrap(NonNull::new_unchecked(cache))
                }
            } else {
                // Safety: the pointer points to valid data
                unsafe {
                    let cache = self.unwrap_non_null(Private).as_ref().linearcache.load(Ordering::Relaxed);
                    debug_assert!(!cache.is_null());
                    SimpleVectorRef::wrap(NonNull::new_unchecked(cache))
                }
            }
        }
    }

    /// The `mt` field.
    #[cfg(feature = "extra-fields")]
    pub fn mt(self) -> Option<ValueRef<'scope, 'static>> {
        // Safety: the pointer points to valid data
        unsafe {
            let mt = self.unwrap_non_null(Private).as_ref().mt;
            let mt = NonNull::new(mt)?;
            Some(ValueRef::wrap(mt.cast()))
        }
    }

    /// Incomplete instantiations of this type.
    #[cfg(feature = "extra-fields")]
    pub fn partial(self) -> Option<ValueRef<'scope, 'static>> {
        // Safety: the pointer points to valid data
        unsafe {
            let partial = self.unwrap_non_null(Private).as_ref().partial;
            let partial = NonNull::new(partial)?;
            Some(ValueRef::wrap(partial.cast()))
        }
    }

    /// The `hash` field.
    pub fn hash(self) -> isize {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().hash }
    }

    /// The `n_uninitialized` field.
    #[cfg(not(feature = "lts"))]
    pub fn n_uninitialized(self) -> i32 {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().n_uninitialized }
    }

    /// The `abstract` field.
    #[cfg(not(feature = "lts"))]
    pub fn abstract_(self) -> bool {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().abstract_() != 0 }
    }

    /// The `mutabl` field.
    #[cfg(not(feature = "lts"))]
    pub fn mutabl(self) -> bool {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().mutabl() != 0 }
    }

    /// The `mayinlinealloc` field.
    #[cfg(not(feature = "lts"))]
    pub fn mayinlinealloc(self) -> bool {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().mayinlinealloc() != 0 }
    }
}

impl<'base> TypeName<'base> {
    /// The typename of the `UnionAll` `Type`.
    pub fn of_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_type_typename), Private) }
    }

    /// The typename of the `DataType` `Tuple`.
    pub fn of_tuple<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_tuple_typename), Private) }
    }

    /// The typename of the `UnionAll` `VecElement`.
    pub fn of_vecelement<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_vecelement_typename), Private) }
    }

    /// The typename of the `UnionAll` `Vararg`.
    #[cfg(feature = "lts")]
    pub fn of_vararg<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_vararg_typename), Private) }
    }

    /// The typename of the `UnionAll` `Array`.
    pub fn of_array<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_array_typename), Private) }
    }

    /// The typename of the `UnionAll` `Ptr`.
    #[cfg(not(feature = "lts"))]
    pub fn of_opaque_closure<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_opaque_closure_typename), Private) }
    }

    /// The typename of the `UnionAll` `Ptr`.
    pub fn of_pointer<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_pointer_typename), Private) }
    }

    /// The typename of the `UnionAll` `LLVMPtr`.
    pub fn of_llvmpointer<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_llvmpointer_typename), Private) }
    }

    /// The typename of the `UnionAll` `NamedTuple`.
    pub fn of_namedtuple<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_namedtuple_typename), Private) }
    }
}

impl_julia_typecheck!(TypeName<'scope>, jl_typename_type, 'scope);
impl_debug!(TypeName<'_>);

impl<'scope> WrapperPriv<'scope, '_> for TypeName<'scope> {
    type Wraps = jl_typename_t;
    type TypeConstructorPriv<'target, 'da> = TypeName<'target>;
    const NAME: &'static str = "TypeName";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

/// A reference to a [`TypeName`] that has not been explicitly rooted.
pub type TypeNameRef<'scope> = Ref<'scope, 'static, TypeName<'scope>>;
impl_valid_layout!(TypeNameRef, TypeName);
impl_ref_root!(TypeName, TypeNameRef, 1);

use crate::memory::target::target_type::TargetType;

/// `TypeName` or `TypeNameRef`, depending on the target type `T`.
pub type TypeNameData<'target, T> = <T as TargetType<'target>>::Data<'static, TypeName<'target>>;

/// `JuliaResult<TypeName>` or `JuliaResultRef<TypeNameRef>`, depending on the target type `T`.
pub type TypeNameResult<'target, T> =
    <T as TargetType<'target>>::Result<'static, TypeName<'target>>;
