//! Managed type for `TypeName`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#L380

use std::{marker::PhantomData, ptr::NonNull};

use cfg_if::cfg_if;
#[julia_version(since = "1.7")]
use jl_sys::jl_opaque_closure_typename;
#[julia_version(until = "1.6")]
use jl_sys::jl_vararg_typename;
use jl_sys::{
    jl_array_typename, jl_llvmpointer_typename, jl_namedtuple_typename, jl_pointer_typename,
    jl_tuple_typename, jl_type_typename, jl_typename_t, jl_typename_type, jl_vecelement_typename,
};
use jlrs_macros::julia_version;

use super::Ref;
use crate::{
    data::{managed::{
        module::Module, private::ManagedPriv, simple_vector::SimpleVector, symbol::Symbol,
    }},
    impl_julia_typecheck,
    memory::target::Target,
    private::Private,
};

cfg_if! {
    if #[cfg(feature = "extra-fields")] {
        use crate::data::managed::{value::Value, simple_vector::{SimpleVectorRef, SimpleVectorData}};
    }
}

#[julia_version(since = "1.7")]
cfg_if! {
    if #[cfg(feature = "extra-fields")] {
        use std::sync::atomic::Ordering;
        use crate::data::managed::value::{ValueData, ValueRef};
    }
}

/// Describes the syntactic structure of a type and stores all data common to different
/// instantiations of the type.
#[derive(Copy, Clone, PartialEq)]
#[repr(transparent)]
pub struct TypeName<'scope>(NonNull<jl_typename_t>, PhantomData<&'scope ()>);

impl<'scope> TypeName<'scope> {
    /*
    inspect(Core.TypeName):

    name: Symbol (const)
    module: Module (const)
    names: Core.SimpleVector (const)
    atomicfields: Ptr{Nothing} (const)
    constfields: Ptr{Nothing} (const)
    wrapper: Type (const)
    Typeofwrapper: Type (mut, atomic)
    cache: Core.SimpleVector (mut, atomic)
    linearcache: Core.SimpleVector (mut, atomic)
    mt: Core.MethodTable (const)
    partial: Any (mut)
    hash: Int64 (const)
    n_uninitialized: Int32 (const)
    flags: UInt8 (const)
    max_methods: UInt8 (mut)
    */

    /// The `name` field.
    pub fn name(self) -> Symbol<'scope> {
        // Safety: the pointer points to valid data
        unsafe {
            let name = self.unwrap_non_null(Private).as_ref().name;
            debug_assert!(!name.is_null());
            Symbol::wrap_non_null(NonNull::new_unchecked(name), Private)
        }
    }

    /// The `module` field.
    pub fn module(self) -> Module<'scope> {
        // Safety: the pointer points to valid data
        unsafe {
            let module = self.unwrap_non_null(Private).as_ref().module;
            debug_assert!(!module.is_null());
            Module::wrap_non_null(NonNull::new_unchecked(module), Private)
        }
    }

    /// Field names.
    pub fn names(self) -> SimpleVector<'scope> {
        // Safety: the pointer points to valid data
        unsafe {
            let names = self.unwrap_non_null(Private).as_ref().names;
            debug_assert!(!names.is_null());
            SimpleVector::wrap_non_null(NonNull::new_unchecked(names), Private)
        }
    }

    #[julia_version(since = "1.7")]
    /// The `atomicfields` field.
    pub fn atomicfields(self) -> *const u32 {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().atomicfields }
    }

    #[julia_version(since = "1.8")]
    /// The `atomicfields` field.
    pub fn constfields(self) -> *const u32 {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().constfields }
    }

    #[cfg(feature = "extra-fields")]
    /// Either the only instantiation of the type (if no parameters) or a `UnionAll` accepting
    /// parameters to make an instantiation.
    pub fn wrapper(self) -> Value<'scope, 'static> {
        // Safety: the pointer points to valid data
        unsafe {
            let wrapper = self.unwrap_non_null(Private).as_ref().wrapper;
            debug_assert!(!wrapper.is_null());
            Value::wrap_non_null(NonNull::new_unchecked(wrapper), Private)
        }
    }

    #[julia_version(since = "1.9")]
    #[cfg(feature = "extra-fields")]
    /// cache for Type{wrapper}
    pub fn typeof_wrapper<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let typeof_wrapper = self
                .unwrap_non_null(Private)
                .as_ref()
                .Typeofwrapper
                .load(Ordering::Relaxed);

            let typeof_wrapper = NonNull::new(typeof_wrapper)?;
            Some(ValueRef::wrap(typeof_wrapper).root(target))
        }
    }

    #[cfg(feature = "extra-fields")]
    #[julia_version(until = "1.6")]
    /// Sorted array.
    pub fn cache<'target, T>(self, target: T) -> SimpleVectorData<'target, T>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let cache = self.unwrap_non_null(Private).as_ref().cache;
            debug_assert!(!cache.is_null());
            SimpleVectorRef::wrap(NonNull::new_unchecked(cache)).root(target)
        }
    }

    #[cfg(feature = "extra-fields")]
    #[julia_version(since = "1.7")]
    /// Sorted array.
    pub fn cache<'target, T>(self, target: T) -> SimpleVectorData<'target, T>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let cache = self
                .unwrap_non_null(Private)
                .as_ref()
                .cache
                .load(Ordering::Relaxed);
            debug_assert!(!cache.is_null());
            SimpleVectorRef::wrap(NonNull::new_unchecked(cache)).root(target)
        }
    }

    #[julia_version(until = "1.6")]
    #[cfg(feature = "extra-fields")]
    /// Unsorted array.
    pub fn linear_cache<'target, T>(self, target: T) -> SimpleVectorData<'target, T>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let cache = self.unwrap_non_null(Private).as_ref().linearcache;
            debug_assert!(!cache.is_null());
            SimpleVectorRef::wrap(NonNull::new_unchecked(cache)).root(target)
        }
    }

    #[julia_version(since = "1.7")]
    #[cfg(feature = "extra-fields")]
    /// Unsorted array.
    pub fn linear_cache<'target, T>(self, target: T) -> SimpleVectorData<'target, T>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let cache = self
                .unwrap_non_null(Private)
                .as_ref()
                .linearcache
                .load(Ordering::Relaxed);
            debug_assert!(!cache.is_null());
            SimpleVectorRef::wrap(NonNull::new_unchecked(cache)).root(target)
        }
    }

    /// The `mt` field.
    #[cfg(feature = "extra-fields")]
    pub fn mt<'target, T>(self, target: T) -> ValueData<'target, 'static, T>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let mt = self.unwrap_non_null(Private).as_ref().mt;
            debug_assert!(!mt.is_null());
            ValueRef::wrap(NonNull::new_unchecked(mt.cast())).root(target)
        }
    }

    /// Incomplete instantiations of this type.
    #[cfg(feature = "extra-fields")]
    pub fn partial<'target, T>(self, target: T) -> Option<ValueData<'target, 'static, T>>
    where
        T: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let partial = self.unwrap_non_null(Private).as_ref().partial;
            let partial = NonNull::new(partial)?;
            Some(ValueRef::wrap(partial.cast()).root(target))
        }
    }

    /// The `hash` field.
    pub fn hash(self) -> isize {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().hash }
    }

    #[julia_version(since = "1.7")]
    /// The `n_uninitialized` field.
    pub fn n_uninitialized(self) -> i32 {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().n_uninitialized }
    }

    #[julia_version(since = "1.7")]
    /// The `abstract` field.
    pub fn is_abstract(self) -> bool {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().abstract_() != 0 }
    }

    #[julia_version(since = "1.7")]
    /// The `mutabl` field.
    pub fn is_mutable(self) -> bool {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().mutabl() != 0 }
    }

    #[julia_version(since = "1.7")]
    /// The `mayinlinealloc` field.
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

    #[julia_version(until = "1.6")]
    /// The typename of the `UnionAll` `Vararg`.
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

    #[julia_version(since = "1.7")]
    /// The typename of the `UnionAll` `Ptr`.
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

impl<'scope> ManagedPriv<'scope, '_> for TypeName<'scope> {
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

impl_construct_type_managed!(TypeName, 1, jl_typename_type);

/// A reference to a [`TypeName`] that has not been explicitly rooted.
pub type TypeNameRef<'scope> = Ref<'scope, 'static, TypeName<'scope>>;

/// A [`TypeNameRef`] with static lifetimes. This is a useful shorthand for signatures of
/// `ccall`able functions that return a [`TypeName`].
pub type TypeNameRet = Ref<'static, 'static, TypeName<'static>>;

impl_valid_layout!(TypeNameRef, TypeName);

use crate::memory::target::target_type::TargetType;

/// `TypeName` or `TypeNameRef`, depending on the target type `T`.
pub type TypeNameData<'target, T> = <T as TargetType<'target>>::Data<'static, TypeName<'target>>;

/// `JuliaResult<TypeName>` or `JuliaResultRef<TypeNameRef>`, depending on the target type `T`.
pub type TypeNameResult<'target, T> =
    <T as TargetType<'target>>::Result<'static, TypeName<'target>>;

impl_ccall_arg_managed!(TypeName, 1);
impl_into_typed!(TypeName);
