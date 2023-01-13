//! A `Value` with typed contents.
//!
//! When exposing functions written in Rust to Julia with the [`julia_module`] macro, the argument
//! and return types of the generated function are generated using the types of the Rust function.
//! By using a `TypedValue` as an argument, that data is guaranteed to be passed to the Rust
//! function as managed data. By using it as a return type, the generated function is guaranteed
//! to return data of that type.

use std::{
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use jl_sys::jl_value_t;

use super::{
    tracked::{Tracked, TrackedMut},
    Value, ValueRef,
};
use crate::{
    convert::{
        ccall_types::{CCallArg, CCallReturn},
        construct_type::ConstructType,
        into_julia::IntoJulia,
    },
    data::{
        layout::valid_layout::{ValidField, ValidLayout},
        managed::{datatype::DataTypeData, private::ManagedPriv, typecheck::Typecheck, Ref},
    },
    memory::target::ExtendedTarget,
    prelude::{JlrsResult, Managed, Target},
    private::Private,
};

/// A `Value` with typed contents.
#[repr(transparent)]
pub struct TypedValue<'scope, 'data, T>(
    NonNull<jl_value_t>,
    PhantomData<NonNull<T>>,
    PhantomData<&'scope ()>,
    PhantomData<&'data ()>,
);

impl<U: ValidLayout + ConstructType + IntoJulia> TypedValue<'_, '_, U> {
    /// Create a new typed value, any type that implements [`IntoJulia`] can be converted using
    /// this function.
    pub fn new<'target, T>(target: T, data: U) -> TypedValueData<'target, 'static, U, T>
    where
        T: Target<'target>,
    {
        unsafe {
            Value::new(&target, data)
                .as_value()
                .cast_unchecked::<TypedValue<U>>()
                .root(target)
        }
    }
}

impl<'scope, 'data, U: ValidLayout + ConstructType> TypedValue<'scope, 'data, U> {
    /// Track `self` immutably.
    ///
    /// See [`Value::track`] for more information.
    pub unsafe fn track<'borrow>(&'borrow self) -> JlrsResult<Tracked<'borrow, 'scope, 'data, U>> {
        self.deref().track()
    }

    /// Track `self` mutably.
    ///
    /// See [`Value::track_mut`] for more information.
    pub unsafe fn track_mut<'borrow>(
        &'borrow mut self,
    ) -> JlrsResult<TrackedMut<'borrow, 'scope, 'data, U>> {
        self.deref_mut().track_mut()
    }
}

impl<'scope, 'data, U: ValidLayout + ConstructType> Deref for TypedValue<'scope, 'data, U> {
    type Target = Value<'scope, 'data>;

    fn deref(&self) -> &Self::Target {
        unsafe { std::mem::transmute(self) }
    }
}
impl<'scope, 'data, U: ValidLayout + ConstructType> DerefMut for TypedValue<'scope, 'data, U> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::mem::transmute(self) }
    }
}

impl<T> Debug for TypedValue<'_, '_, T>
where
    T: ValidLayout + ConstructType,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{:?}", self.as_value())
    }
}

impl<T> Clone for TypedValue<'_, '_, T>
where
    T: ValidLayout + ConstructType,
{
    fn clone(&self) -> Self {
        unsafe { Self::wrap_non_null(self.unwrap_non_null(Private), Private) }
    }
}

impl<T> Copy for TypedValue<'_, '_, T> where T: ValidLayout + ConstructType {}

unsafe impl<T> ValidLayout for TypedValue<'_, '_, T>
where
    T: ValidLayout + ConstructType,
{
    fn valid_layout(v: Value) -> bool {
        ValueRef::valid_layout(v)
    }

    const IS_REF: bool = true;
}
unsafe impl<T> ValidField for Option<TypedValue<'_, '_, T>>
where
    T: ValidLayout + ConstructType,
{
    fn valid_field(v: Value) -> bool {
        Option::<ValueRef>::valid_field(v)
    }
}

pub type TypedValueRef<'scope, 'data, T> = Ref<'scope, 'data, TypedValue<'scope, 'data, T>>;

/// A [`TypedValueRef`] with static lifetimes. This is a useful shorthand for signatures of
/// `ccall`able functions that return a [`TypedValue`].
pub type TypedValueRet<T> = Ref<'static, 'static, TypedValue<'static, 'static, T>>;

impl<'scope, 'data, T> ManagedPriv<'scope, 'data> for TypedValue<'scope, 'data, T>
where
    T: ValidLayout + ConstructType,
{
    type Wraps = jl_value_t;
    type TypeConstructorPriv<'target, 'da> = TypedValue<'target, 'da, T>;
    const NAME: &'static str = "TypedValue";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData, PhantomData, PhantomData)
    }

    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

unsafe impl<'scope, 'data, T> Typecheck for TypedValue<'scope, 'data, T>
where
    T: ValidLayout + ConstructType,
{
    fn typecheck(t: crate::prelude::DataType) -> bool {
        T::valid_layout(t.as_value())
    }
}

unsafe impl<U> ConstructType for Option<TypedValueRef<'_, 'static, U>>
where
    U: ValidLayout + ConstructType,
{
    fn base_type<'target, T>(target: &T) -> Value<'target, 'static>
    where
        T: Target<'target>,
    {
        U::base_type(target)
    }

    fn construct_type<'target, 'current, 'borrow, T>(
        target: ExtendedTarget<'target, 'current, 'borrow, T>,
    ) -> DataTypeData<'target, T>
    where
        T: Target<'target>,
    {
        U::construct_type(target)
    }
}

use crate::memory::target::target_type::TargetType;

/// `TypedValue` or `TypedValueRef`, depending on the target type `T`.
pub type TypedValueData<'target, 'data, U, T> =
    <T as TargetType<'target>>::Data<'data, TypedValue<'target, 'data, U>>;

/// `JuliaResult<TypedValue>` or `JuliaResultRef<TypedValueRef>`, depending on the target type
/// `T`.
pub type TypedValueResult<'target, 'data, U, T> =
    <T as TargetType<'target>>::Result<'data, TypedValue<'target, 'data, U>>;

unsafe impl<'scope, 'data, T: ConstructType + ValidLayout> CCallArg
    for TypedValue<'scope, 'data, T>
{
    type CCallArgType = Option<ValueRef<'scope, 'data>>;
    type FunctionArgType = T;
}

unsafe impl<T: ConstructType + ValidLayout> CCallReturn for TypedValueRef<'static, 'static, T> {
    type CCallReturnType = Option<ValueRef<'static, 'static>>;
    type FunctionReturnType = T;
}
