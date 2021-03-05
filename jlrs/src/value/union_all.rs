//! Support for values with the `Core.UnionAll` type.

use super::type_var::TypeVar;
use super::Value;
use crate::convert::cast::Cast;
use crate::error::{JlrsError, JlrsResult};
use crate::memory::global::Global;
use crate::value::datatype::DataType;
use crate::{impl_julia_type, impl_julia_typecheck, impl_valid_layout};
use jl_sys::{
    jl_abstractarray_type, jl_anytuple_type_type, jl_array_type, jl_densearray_type,
    jl_llvmpointer_type, jl_namedtuple_type, jl_pointer_type, jl_ref_type, jl_type_type,
    jl_unionall_t, jl_unionall_type, jl_vararg_type,
};
use std::{fmt::{Debug, Formatter, Result as FmtResult}, marker::PhantomData};

/// An iterated union of types. If a struct field has a parametric type with some of its
/// parameters unknown, its type is represented by a `UnionAll`.
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct UnionAll<'frame>(*mut jl_unionall_t, PhantomData<&'frame ()>);

impl<'frame> UnionAll<'frame> {
    pub(crate) unsafe fn wrap(union_all: *mut jl_unionall_t) -> Self {
        UnionAll(union_all, PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn ptr(self) -> *mut jl_unionall_t {
        self.0
    }

    /// The type at the bottom of this `UnionAll`.
    pub fn base_type(self) -> DataType<'frame> {
        let mut b = self;
        while b.body().is::<UnionAll>() {
            unsafe {
                b = Value::from(b.body()).cast_unchecked::<UnionAll>();
            }
        }

        Value::from(b.body()).cast::<DataType>().unwrap()
    }

    /// The body of this `UnionAll`. This is either another `UnionAll` or a `DataType`.
    pub fn body(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap((&*self.ptr()).body) }
    }

    /// The type variable associated with this "layer" of the `UnionAll`.
    pub fn var(self) -> TypeVar<'frame> {
        unsafe { TypeVar::wrap((&*self.ptr()).var) }
    }

    /// Convert `self` to a `Value`.
    pub fn as_value(self) -> Value<'frame, 'static> {
        self.into()
    }
}

impl<'base> UnionAll<'base> {
    /// The `UnionAll` `Type`.
    pub fn type_type(_: Global<'base>) -> Self {
        unsafe { UnionAll::wrap(jl_type_type) }
    }

    /// `Type{T} where T<:Tuple`
    pub fn anytuple_type_type(_: Global<'base>) -> Self {
        unsafe { UnionAll::wrap(jl_anytuple_type_type) }
    }

    /// The `UnionAll` `Vararg`.
    pub fn vararg_type(_: Global<'base>) -> Self {
        unsafe { UnionAll::wrap(jl_vararg_type) }
    }

    /// The `UnionAll` `AbstractArray`.
    pub fn abstractarray_type(_: Global<'base>) -> Self {
        unsafe { UnionAll::wrap(jl_abstractarray_type) }
    }

    /// The `UnionAll` `DenseArray`.
    pub fn densearray_type(_: Global<'base>) -> Self {
        unsafe { UnionAll::wrap(jl_densearray_type) }
    }

    /// The `UnionAll` `Array`.
    pub fn array_type(_: Global<'base>) -> Self {
        unsafe { UnionAll::wrap(jl_array_type) }
    }

    /// The `UnionAll` `Ptr`.
    pub fn pointer_type(_: Global<'base>) -> Self {
        unsafe { UnionAll::wrap(jl_pointer_type) }
    }

    /// The `UnionAll` `LLVMPtr`.
    pub fn llvmpointer_type(_: Global<'base>) -> Self {
        unsafe { UnionAll::wrap(jl_llvmpointer_type) }
    }

    /// The `UnionAll` `Ref`.
    pub fn ref_type(_: Global<'base>) -> Self {
        unsafe { UnionAll::wrap(jl_ref_type) }
    }

    /// The `UnionAll` `NamedTuple`.
    pub fn namedtuple_type(_: Global<'base>) -> Self {
        unsafe { UnionAll::wrap(jl_namedtuple_type) }
    }
}

impl<'scope> Debug for UnionAll<'scope> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("UnionAll").finish()
    }
}

impl<'frame> Into<Value<'frame, 'static>> for UnionAll<'frame> {
    fn into(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap(self.ptr().cast()) }
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for UnionAll<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotAUnionAll)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        Self::wrap(value.ptr().cast())
    }
}

impl_julia_typecheck!(UnionAll<'frame>, jl_unionall_type, 'frame);
impl_julia_type!(UnionAll<'frame>, jl_unionall_type, 'frame);
impl_valid_layout!(UnionAll<'frame>, 'frame);
