//! Wrapper for `UnionAll`, A union of types over all values of a type parameter.

use super::type_var::TypeVar;
use super::{private::Wrapper, value::Value};
#[cfg(not(all(target_os = "windows", feature = "lts")))]
use crate::error::{JuliaResult, JuliaResultRef};
use crate::impl_debug;
use crate::wrappers::ptr::datatype::DataType;
use crate::wrappers::ptr::DataTypeRef;
use crate::wrappers::ptr::{TypeVarRef, ValueRef};
use crate::{
    error::JlrsResult,
    memory::{output::Output, scope::PartialScope},
};
use crate::{impl_julia_typecheck, impl_valid_layout};
use crate::{memory::global::Global, private::Private};

use jl_sys::{
    jl_abstractarray_type, jl_anytuple_type_type, jl_array_type, jl_densearray_type,
    jl_llvmpointer_type, jl_namedtuple_type, jl_pointer_type, jl_ref_type, jl_type_type,
    jl_type_unionall, jl_unionall_t, jl_unionall_type,
};

#[cfg(not(all(target_os = "windows", feature = "lts")))]
use jl_sys::{jlrs_result_tag_t_JLRS_RESULT_ERR, jlrs_type_unionall};

#[cfg(not(feature = "lts"))]
use jl_sys::jl_opaque_closure_type;
#[cfg(feature = "lts")]
use jl_sys::jl_vararg_type;

use std::{marker::PhantomData, ptr::NonNull};

/// An iterated union of types. If a struct field has a parametric type with some of its
/// parameters unknown, its type is represented by a `UnionAll`.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct UnionAll<'scope>(NonNull<jl_unionall_t>, PhantomData<&'scope ()>);

impl<'scope> UnionAll<'scope> {
    /// Create a new `UnionAll`. If an exception is thrown, it's caught and returned.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub fn new<'target, S>(
        scope: S,
        tvar: TypeVar,
        body: Value<'_, 'static>,
    ) -> JlrsResult<JuliaResult<'target, 'static>>
    where
        S: PartialScope<'target>,
    {
        unsafe {
            let ua = jlrs_type_unionall(tvar.unwrap(Private), body.unwrap(Private));
            if ua.flag == jlrs_result_tag_t_JLRS_RESULT_ERR {
                scope.call_result(Err(NonNull::new_unchecked(ua.data)), Private)
            } else {
                scope.call_result(Ok(NonNull::new_unchecked(ua.data)), Private)
            }
        }
    }

    /// Create a new `UnionAll`. If an exception is thrown it isn't caught
    ///
    /// Safety: an exception must not be thrown if this method is called from a `ccall`ed
    /// function.
    pub unsafe fn new_unchecked<'target, S>(
        scope: S,
        tvar: TypeVar,
        body: Value<'_, 'static>,
    ) -> JlrsResult<Value<'target, 'static>>
    where
        S: PartialScope<'target>,
    {
        let ua = jl_type_unionall(tvar.unwrap(Private), body.unwrap(Private));
        scope.value(NonNull::new_unchecked(ua), Private)
    }

    /// Create a new `UnionAll`. Unlike [`UnionAll::new`] this method doesn't root the allocated
    /// value or exception.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub fn new_unrooted<'global>(
        _: Global<'global>,
        tvar: TypeVar,
        body: Value<'_, 'static>,
    ) -> JuliaResultRef<'global, 'static> {
        unsafe {
            let ua = jlrs_type_unionall(tvar.unwrap(Private), body.unwrap(Private));
            if ua.flag == jlrs_result_tag_t_JLRS_RESULT_ERR {
                Err(ValueRef::wrap(ua.data))
            } else {
                Ok(ValueRef::wrap(ua.data))
            }
        }
    }

    /// Create a new `UnionAll`. Unlike [`UnionAll::new_unchecked`] this method doesn't root the
    /// allocated value. If an exception is thrown it isn't caught
    ///
    /// Safety: an exception must not be thrown if this method is called from a `ccall`ed
    /// function.
    pub unsafe fn new_unrooted_unchecked<'global>(
        _: Global<'global>,
        tvar: TypeVar,
        body: Value<'_, 'static>,
    ) -> ValueRef<'global, 'static> {
        let ua = jl_type_unionall(tvar.unwrap(Private), body.unwrap(Private));
        ValueRef::wrap(ua)
    }

    /// The type at the bottom of this `UnionAll`.
    pub fn base_type(self) -> DataTypeRef<'scope> {
        let mut b = self;
        unsafe {
            while b.body().value_unchecked().is::<UnionAll>() {
                b = b.body().value_unchecked().cast_unchecked::<UnionAll>();
            }
        }

        unsafe {
            DataTypeRef::wrap(
                b.body()
                    .value_unchecked()
                    .cast::<DataType>()
                    .unwrap()
                    .unwrap(Private),
            )
        }
    }

    /*
    for (a,b) in zip(fieldnames(UnionAll), fieldtypes(UnionAll))
        println(a,": ", b)
    end
    var: TypeVar
    body: Any
    */

    /// The body of this `UnionAll`. This is either another `UnionAll` or a `DataType`.
    pub fn body(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().body) }
    }

    /// The type variable associated with this "layer" of the `UnionAll`.
    pub fn var(self) -> TypeVarRef<'scope> {
        unsafe { TypeVarRef::wrap(self.unwrap_non_null(Private).as_ref().var) }
    }

    /// Use the `Output` to extend the lifetime of this data.
    pub fn root<'target>(self, output: Output<'target>) -> UnionAll<'target> {
        unsafe {
            let ptr = self.unwrap_non_null(Private);
            output.set_root::<UnionAll>(ptr);
            UnionAll::wrap_non_null(ptr, Private)
        }
    }
}

impl<'base> UnionAll<'base> {
    /// The `UnionAll` `Type`.
    pub fn type_type(_: Global<'base>) -> Self {
        unsafe { UnionAll::wrap(jl_type_type, Private) }
    }

    /// `Type{T} where T<:Tuple`
    pub fn anytuple_type_type(_: Global<'base>) -> Self {
        unsafe { UnionAll::wrap(jl_anytuple_type_type, Private) }
    }

    /// The `UnionAll` `Vararg`.
    #[cfg(feature = "lts")]
    pub fn vararg_type(_: Global<'base>) -> Self {
        unsafe { UnionAll::wrap(jl_vararg_type, Private) }
    }

    /// The `UnionAll` `AbstractArray`.
    pub fn abstractarray_type(_: Global<'base>) -> Self {
        unsafe { UnionAll::wrap(jl_abstractarray_type, Private) }
    }

    /// The `UnionAll` `OpaqueClosure`.
    #[cfg(not(feature = "lts"))]
    pub fn opaque_closure_type(_: Global<'base>) -> Self {
        unsafe { UnionAll::wrap(jl_opaque_closure_type, Private) }
    }

    /// The `UnionAll` `DenseArray`.
    pub fn densearray_type(_: Global<'base>) -> Self {
        unsafe { UnionAll::wrap(jl_densearray_type, Private) }
    }

    /// The `UnionAll` `Array`.
    pub fn array_type(_: Global<'base>) -> Self {
        unsafe { UnionAll::wrap(jl_array_type, Private) }
    }

    /// The `UnionAll` `Ptr`.
    pub fn pointer_type(_: Global<'base>) -> Self {
        unsafe { UnionAll::wrap(jl_pointer_type, Private) }
    }

    /// The `UnionAll` `LLVMPtr`.
    pub fn llvmpointer_type(_: Global<'base>) -> Self {
        unsafe { UnionAll::wrap(jl_llvmpointer_type, Private) }
    }

    /// The `UnionAll` `Ref`.
    pub fn ref_type(_: Global<'base>) -> Self {
        unsafe { UnionAll::wrap(jl_ref_type, Private) }
    }

    /// The `UnionAll` `NamedTuple`.
    pub fn namedtuple_type(_: Global<'base>) -> Self {
        unsafe { UnionAll::wrap(jl_namedtuple_type, Private) }
    }
}

impl_julia_typecheck!(UnionAll<'scope>, jl_unionall_type, 'scope);
impl_debug!(UnionAll<'_>);
impl_valid_layout!(UnionAll<'scope>, 'scope);

impl<'scope> Wrapper<'scope, '_> for UnionAll<'scope> {
    type Wraps = jl_unionall_t;
    const NAME: &'static str = "UnionAll";

    #[inline(always)]
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    #[inline(always)]
    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

impl_root!(UnionAll, 1);
