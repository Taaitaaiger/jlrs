//! A `Value` annotated with its type constructor
//!
//! When a Rust function is exported to Julia with the [`julia_module`] macro, the generated Julia
//! function looks like this:
//!
//! ```julia
//! function fn_name(arg1::FnArg1, arg2::FnArg2, ...)::FnRet
//!     ccall(fn_ptr, CCallRet, (CCallArg1, CCallArg2, ...), arg1, arg2, ...)
//! end
//! ```
//!
//! The argument and return types are generated from the signature of the exported function. When
//! `TypedValue<Ty>` is used as an argument, the `CCallArg` is `Any` and the `FnArg` is the type
//! that is constructed from `Ty`. The same is true for `CCallRet` and `FnRet` when
//! `TypedValueRet<Ty>` is returned.
//!
//! [`julia_module`]: ::jlrs_macros::julia_module

use std::{
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use jl_sys::jl_value_t;

use super::{
    tracked::{Tracked, TrackedMut},
    Value, ValueData, ValueRef,
};
use crate::{
    convert::{
        ccall_types::{CCallArg, CCallReturn},
        into_julia::IntoJulia,
    },
    data::{
        layout::valid_layout::{ValidField, ValidLayout},
        managed::{private::ManagedPriv, Ref},
        types::{construct_type::ConstructType, typecheck::Typecheck},
    },
    error::TypeError,
    memory::target::{frame::GcFrame, ExtendedTarget},
    prelude::{JlrsResult, Managed, Target},
    private::Private,
};

/// Convert managed data to a `TypedValue`.
pub trait AsTyped<'scope, 'data>: Managed<'scope, 'data> {
    fn as_typed(self) -> JlrsResult<TypedValue<'scope, 'data, Self>>;
}

/// A `Value` and its type constructor.
#[repr(transparent)]
pub struct TypedValue<'scope, 'data, T>(
    NonNull<jl_value_t>,
    PhantomData<NonNull<T>>,
    PhantomData<&'scope ()>,
    PhantomData<&'data ()>,
);

impl<U: ConstructType + IntoJulia> TypedValue<'_, '_, U> {
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

impl<'scope, 'data, U: ConstructType> TypedValue<'scope, 'data, U> {
    /// Create a new typed value from an existing value.
    pub fn from_value(
        frame: &mut GcFrame,
        value: Value<'scope, 'data>,
    ) -> JlrsResult<TypedValue<'scope, 'data, U>> {
        frame.scope(|mut frame| {
            let ty = U::construct_type(frame.as_extended_target());
            if value.isa(ty) {
                unsafe {
                    Ok(TypedValue::<U>::wrap_non_null(
                        value.unwrap_non_null(Private),
                        Private,
                    ))
                }
            } else {
                Err(TypeError::NotA {
                    value: value.display_string_or("<Cannot display value>"),
                    field_type: ty.display_string_or("<Cannot display type>"),
                })?
            }
        })
    }

    /// Create a new typed value from an existing value without checking the value is an instance
    /// of `U`.
    ///
    /// Safety: `value` must be an instance of the constructed type `U`.
    pub unsafe fn from_value_unchecked(
        value: Value<'scope, 'data>,
    ) -> TypedValue<'scope, 'data, U> {
        TypedValue::<U>::wrap_non_null(value.unwrap_non_null(Private), Private)
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
    T: ConstructType,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{:?}", self.as_value())
    }
}

impl<T> Clone for TypedValue<'_, '_, T>
where
    T: ConstructType,
{
    fn clone(&self) -> Self {
        unsafe { Self::wrap_non_null(self.unwrap_non_null(Private), Private) }
    }
}

impl<T> Copy for TypedValue<'_, '_, T> where T: ConstructType {}

unsafe impl<T> ValidLayout for TypedValue<'_, '_, T>
where
    T: ConstructType,
{
    fn valid_layout(v: Value) -> bool {
        ValueRef::valid_layout(v)
    }

    const IS_REF: bool = true;
}
unsafe impl<T> ValidField for Option<TypedValue<'_, '_, T>>
where
    T: ConstructType,
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
    T: ConstructType,
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

unsafe impl<U> ConstructType for TypedValue<'_, 'static, U>
where
    U: ConstructType,
{
    fn construct_type<'target, 'current, 'borrow, T>(
        target: ExtendedTarget<'target, '_, '_, T>,
    ) -> ValueData<'target, 'static, T>
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

unsafe impl<'scope, 'data, T: ConstructType> CCallArg for TypedValue<'scope, 'data, T> {
    type CCallArgType = Value<'scope, 'data>;
    type FunctionArgType = T;
}

unsafe impl<T: ConstructType> CCallReturn for TypedValueRef<'static, 'static, T> {
    type CCallReturnType = Value<'static, 'static>;
    type FunctionReturnType = T;
}
