//! Managed type for `TypeName`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#L380

use std::{marker::PhantomData, ptr::NonNull};

#[julia_version(until = "1.6")]
use jl_sys::jl_vararg_typename;
use jl_sys::{
    jl_array_typename, jl_llvmpointer_typename, jl_namedtuple_typename, jl_pointer_typename,
    jl_tuple_typename, jl_type_typename, jl_typename_t, jl_typename_type, jl_vecelement_typename,
    jlrs_typename_module, jlrs_typename_name, jlrs_typename_names, jlrs_typename_wrapper,
};
use jlrs_macros::julia_version;

use super::{simple_vector::SimpleVector, value::Value, Ref};
use crate::{
    data::managed::{module::Module, private::ManagedPriv, symbol::Symbol},
    impl_julia_typecheck,
    memory::target::{Target, TargetResult},
    private::Private,
};

/// Describes the syntactic structure of a type and stores all data common to different
/// instantiations of the type.
#[derive(Copy, Clone, PartialEq)]
#[repr(transparent)]
pub struct TypeName<'scope>(NonNull<jl_typename_t>, PhantomData<&'scope ()>);

impl<'scope> TypeName<'scope> {
    /// The `name` field.
    #[inline]
    pub fn name(self) -> Symbol<'scope> {
        // Safety: the pointer points to valid data
        unsafe {
            let name = jlrs_typename_name(self.unwrap(Private));
            Symbol::wrap_non_null(NonNull::new_unchecked(name), Private)
        }
    }

    /// The `name` field.
    #[inline]
    pub fn names(self) -> SimpleVector<'scope> {
        // Safety: the pointer points to valid data
        unsafe {
            let names = jlrs_typename_names(self.unwrap(Private));
            SimpleVector::wrap_non_null(NonNull::new_unchecked(names), Private)
        }
    }

    /// The `module` field.
    #[inline]
    pub fn module(self) -> Module<'scope> {
        // Safety: the pointer points to valid data
        unsafe {
            let module = jlrs_typename_module(self.unwrap(Private));
            Module::wrap_non_null(NonNull::new_unchecked(module), Private)
        }
    }

    /// The `module` field.
    #[inline]
    pub fn wrapper(self) -> Value<'scope, 'static> {
        // Safety: the pointer points to valid data
        unsafe {
            let module = jlrs_typename_wrapper(self.unwrap(Private));
            Value::wrap_non_null(NonNull::new_unchecked(module), Private)
        }
    }

    #[julia_version(since = "1.7")]
    /// The `atomicfields` field.
    #[inline]
    pub fn atomicfields(self) -> *const u32 {
        // Safety: the pointer points to valid data
        unsafe { jl_sys::jlrs_typename_atomicfields(self.unwrap(Private)) }
    }

    #[julia_version(since = "1.8")]
    /// The `atomicfields` field.
    #[inline]
    pub fn constfields(self) -> *const u32 {
        // Safety: the pointer points to valid data
        unsafe { jl_sys::jlrs_typename_constfields(self.unwrap(Private)) }
    }

    #[julia_version(since = "1.7")]
    /// The `abstract` field.
    #[inline]
    pub fn is_abstract(self) -> bool {
        // Safety: the pointer points to valid data
        unsafe { jl_sys::jlrs_typename_abstract(self.unwrap(Private)) != 0 }
    }

    #[julia_version(since = "1.7")]
    /// The `mutabl` field.
    #[inline]
    pub fn is_mutable(self) -> bool {
        // Safety: the pointer points to valid data
        unsafe { jl_sys::jlrs_typename_mutable(self.unwrap(Private)) != 0 }
    }

    #[julia_version(since = "1.7")]
    /// The `mayinlinealloc` field.
    #[inline]
    pub fn mayinlinealloc(self) -> bool {
        // Safety: the pointer points to valid data
        unsafe { jl_sys::jlrs_typename_mayinlinealloc(self.unwrap(Private)) != 0 }
    }
}

impl<'base> TypeName<'base> {
    /// The typename of the `UnionAll` `Type`.
    #[inline]
    pub fn of_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_type_typename), Private) }
    }

    /// The typename of the `DataType` `Tuple`.
    #[inline]
    pub fn of_tuple<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_tuple_typename), Private) }
    }

    /// The typename of the `UnionAll` `VecElement`.
    #[inline]
    pub fn of_vecelement<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_vecelement_typename), Private) }
    }

    #[julia_version(until = "1.6")]
    /// The typename of the `UnionAll` `Vararg`.
    #[inline]
    pub fn of_vararg<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_vararg_typename), Private) }
    }

    /// The typename of the `UnionAll` `Array`.
    #[inline]
    pub fn of_array<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_array_typename), Private) }
    }

    /// The typename of the `UnionAll` `Ptr`.
    #[inline]
    pub fn of_pointer<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_pointer_typename), Private) }
    }

    /// The typename of the `UnionAll` `LLVMPtr`.
    #[inline]
    pub fn of_llvmpointer<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_llvmpointer_typename), Private) }
    }

    /// The typename of the `UnionAll` `NamedTuple`.
    #[inline]
    pub fn of_namedtuple<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_namedtuple_typename), Private) }
    }

    #[julia_version(since = "1.11")]
    /// The typename of the `UnionAll` `GenericMemory`.
    #[inline]
    pub fn of_genericmemory<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'base>,
    {
        // Safety: global constant
        unsafe {
            Self::wrap_non_null(
                NonNull::new_unchecked(jl_sys::jl_genericmemory_typename),
                Private,
            )
        }
    }

    #[julia_version(since = "1.11")]
    /// The typename of the `UnionAll` `GenericMemoryRef`.
    #[inline]
    pub fn of_genericmemoryref<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'base>,
    {
        // Safety: global constant
        unsafe {
            Self::wrap_non_null(
                NonNull::new_unchecked(jl_sys::jl_genericmemoryref_typename),
                Private,
            )
        }
    }
}

impl_julia_typecheck!(TypeName<'scope>, jl_typename_type, 'scope);
impl_debug!(TypeName<'_>);

impl<'scope> ManagedPriv<'scope, '_> for TypeName<'scope> {
    type Wraps = jl_typename_t;
    type WithLifetimes<'target, 'da> = TypeName<'target>;
    const NAME: &'static str = "TypeName";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    #[inline]
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    #[inline]
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

impl_valid_layout!(TypeNameRef, TypeName, jl_typename_type);

use crate::memory::target::TargetType;

/// `TypeName` or `TypeNameRef`, depending on the target type `Tgt`.
pub type TypeNameData<'target, Tgt> =
    <Tgt as TargetType<'target>>::Data<'static, TypeName<'target>>;

/// `JuliaResult<TypeName>` or `JuliaResultRef<TypeNameRef>`, depending on the target type `Tgt`.
pub type TypeNameResult<'target, Tgt> = TargetResult<'target, 'static, TypeName<'target>, Tgt>;

impl_ccall_arg_managed!(TypeName, 1);
impl_into_typed!(TypeName);
