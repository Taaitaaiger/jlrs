//! Wrapper for `Core.UnionAll`.

use super::{private::Wrapper, value::Value};
use crate::wrappers::ptr::datatype::DataType;
use crate::wrappers::ptr::DataTypeRef;
use crate::wrappers::ptr::{TypeVarRef, ValueRef};
use crate::{impl_julia_typecheck, impl_valid_layout};
use crate::{memory::global::Global, private::Private};
use jl_sys::{
    jl_abstractarray_type, jl_anytuple_type_type, jl_array_type, jl_densearray_type,
    jl_llvmpointer_type, jl_namedtuple_type, jl_pointer_type, jl_ref_type, jl_type_type,
    jl_unionall_t, jl_unionall_type, jl_vararg_type,
};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    ptr::NonNull,
};

/// An iterated union of types. If a struct field has a parametric type with some of its
/// parameters unknown, its type is represented by a `UnionAll`.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct UnionAll<'frame>(NonNull<jl_unionall_t>, PhantomData<&'frame ()>);

impl<'frame> UnionAll<'frame> {
    /// The type at the bottom of this `UnionAll`.
    pub fn base_type(self) -> DataTypeRef<'frame> {
        let mut b = self;
        unsafe {
            while b.body().value_unchecked().is::<UnionAll>() {
                b = Value::from(b.body().value_unchecked()).cast_unchecked::<UnionAll>();
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
    pub fn body(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().body) }
    }

    /// The type variable associated with this "layer" of the `UnionAll`.
    pub fn var(self) -> TypeVarRef<'frame> {
        unsafe { TypeVarRef::wrap(self.unwrap_non_null(Private).as_ref().var) }
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
    pub fn vararg_type(_: Global<'base>) -> Self {
        unsafe { UnionAll::wrap(jl_vararg_type, Private) }
    }

    /// The `UnionAll` `AbstractArray`.
    pub fn abstractarray_type(_: Global<'base>) -> Self {
        unsafe { UnionAll::wrap(jl_abstractarray_type, Private) }
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

impl<'scope> Debug for UnionAll<'scope> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("UnionAll").finish()
    }
}

impl_julia_typecheck!(UnionAll<'frame>, jl_unionall_type, 'frame);

impl_valid_layout!(UnionAll<'frame>, 'frame);

impl<'scope> Wrapper<'scope, '_> for UnionAll<'scope> {
    type Internal = jl_unionall_t;

    unsafe fn wrap_non_null(inner: NonNull<Self::Internal>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    unsafe fn unwrap_non_null(self, _: Private) -> NonNull<Self::Internal> {
        self.0
    }
}
