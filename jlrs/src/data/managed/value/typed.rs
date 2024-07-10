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
        managed::{datatype::DataType, private::ManagedPriv, Managed, Ref},
        types::{
            abstract_type::AnyType,
            construct_type::{ArrayTypeConstructor, ConstantIsize, ConstructType},
            typecheck::Typecheck,
        },
    },
    error::{JlrsResult, TypeError},
    memory::{
        scope::LocalScope,
        target::{Target, TargetResult},
    },
    prelude::{TypedArray, TypedRankedArray},
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
    PhantomData<T>,
    PhantomData<&'scope ()>,
    PhantomData<&'data ()>,
);

impl<U: ConstructType + IntoJulia> TypedValue<'_, '_, U> {
    /// Create a new typed value, any type that implements [`IntoJulia`] can be converted using
    /// this function.
    #[inline]
    pub fn new<'target, Tgt>(target: Tgt, data: U) -> TypedValueData<'target, 'static, U, Tgt>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            Value::new(&target, data)
                .as_value()
                .cast_unchecked::<TypedValue<U>>()
                .root(target)
        }
    }
}

impl<U: ConstructType> TypedValue<'_, '_, U> {
    /// Create a new typed value, any type that implements [`ValidLayout`] can be converted using
    /// this function as long as it's valid for `U`.
    #[inline]
    pub fn try_new_with<'target, L, Tgt>(
        target: Tgt,
        data: L,
    ) -> JlrsResult<TypedValueData<'target, 'static, U, Tgt>>
    where
        L: ValidLayout,
        Tgt: Target<'target>,
    {
        unsafe {
            let v = Value::try_new_with::<U, _, _>(&target, data)?.as_value();
            Ok(TypedValue::<U>::from_value_unchecked(v).root(target))
        }
    }
}

impl<'scope, 'data, U: ConstructType> TypedValue<'scope, 'data, U> {
    /// Create a new typed value from an existing value.
    pub fn from_value<'target, Tgt>(
        target: &Tgt,
        value: Value<'scope, 'data>,
    ) -> JlrsResult<TypedValue<'scope, 'data, U>>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            target.local_scope::<_, 1>(|mut frame| {
                let ty = U::construct_type(&mut frame).as_value();
                if value.isa(ty) {
                    Ok(TypedValue::<U>::wrap_non_null(
                        value.unwrap_non_null(Private),
                        Private,
                    ))
                } else {
                    Err(TypeError::NotA {
                        value: value.display_string_or("<Cannot display value>"),
                        field_type: ty.display_string_or("<Cannot display type>"),
                    })?
                }
            })
        }
    }

    /// Create a new typed value from an existing value without checking the value is an instance
    /// of `U`.
    ///
    /// Safety: `value` must be an instance of the constructed type `U`.
    #[inline]
    pub unsafe fn from_value_unchecked(
        value: Value<'scope, 'data>,
    ) -> TypedValue<'scope, 'data, U> {
        TypedValue::<U>::wrap_non_null(value.unwrap_non_null(Private), Private)
    }
}

impl<'scope, 'data, U: ValidLayout + ConstructType> TypedValue<'scope, 'data, U> {
    /// Track `self` immutably.
    ///
    /// See [`Value::track_shared`] for more information.
    #[inline]
    pub unsafe fn track_shared<'tracked>(
        &'tracked self,
    ) -> JlrsResult<Tracked<'tracked, 'scope, 'data, U>> {
        self.deref().track_shared()
    }

    /// Track `self` mutably.
    ///
    /// See [`Value::track_exclusive`] for more information.
    #[inline]
    pub unsafe fn track_exclusive<'tracked>(
        &'tracked mut self,
    ) -> JlrsResult<TrackedMut<'tracked, 'scope, 'data, U>> {
        self.deref_mut().track_exclusive()
    }
}

impl<'scope, 'data, U: ConstructType> TypedValue<'scope, 'data, U> {
    /// Track `self` immutably.
    ///
    /// See [`Value::track_shared`] for more information.
    #[inline]
    pub unsafe fn track_shared_as<'tracked, V: ValidLayout>(
        &'tracked self,
    ) -> JlrsResult<Tracked<'tracked, 'scope, 'data, V>> {
        self.deref().track_shared()
    }

    /// Track `self` mutably.
    ///
    /// See [`Value::track_exclusive`] for more information.
    #[inline]
    pub unsafe fn track_exclusive_as<'tracked, V: ValidLayout>(
        &'tracked mut self,
    ) -> JlrsResult<TrackedMut<'tracked, 'scope, 'data, V>> {
        self.deref_mut().track_exclusive()
    }
}

impl<'scope, 'data, T: ConstructType, const N: isize>
    TypedValue<'scope, 'data, ArrayTypeConstructor<T, ConstantIsize<N>>>
{
    /// Convert `self` to the equivalent `TypedRankedArray` type.
    #[inline]
    pub fn as_typed_ranked_array(self) -> TypedRankedArray<'scope, 'data, T, N> {
        unsafe { std::mem::transmute(self) }
    }
}

impl<'scope, 'data, T: ConstructType, N: ConstructType>
    TypedValue<'scope, 'data, ArrayTypeConstructor<T, N>>
{
    /// Convert `self` to the equivalent `TypedArray` type.
    #[inline]
    pub fn as_typed_array(self) -> TypedArray<'scope, 'data, T> {
        unsafe { std::mem::transmute(self) }
    }
}

impl<U: ConstructType + ValidLayout + Send> TypedValueUnbound<U> {
    /// Track `self` immutably.
    ///
    /// See [`Value::track_shared_unbound`] for more information.
    #[inline]
    pub unsafe fn track_shared_unbound(self) -> JlrsResult<Tracked<'static, 'static, 'static, U>> {
        self.as_value().track_shared_unbound()
    }

    /// Track `self` mutably.
    ///
    /// See [`Value::track_exclusive_unbound`] for more information.
    #[inline]
    pub unsafe fn track_exclusive_unbound(
        self,
    ) -> JlrsResult<TrackedMut<'static, 'static, 'static, U>> {
        self.as_value().track_exclusive_unbound()
    }
}

impl<U: ConstructType> TypedValueUnbound<U> {
    /// Track `self` immutably.
    ///
    /// See [`Value::track_shared_unbound`] for more information.
    #[inline]
    pub unsafe fn track_shared_unbound_as<V: ValidLayout + Send>(
        self,
    ) -> JlrsResult<Tracked<'static, 'static, 'static, V>> {
        self.as_value().track_shared_unbound()
    }

    /// Track `self` mutably.
    ///
    /// See [`Value::track_exclusive_unbound`] for more information.
    #[inline]
    pub unsafe fn track_exclusive_unbound_as<V: ValidLayout + Send>(
        self,
    ) -> JlrsResult<TrackedMut<'static, 'static, 'static, V>> {
        self.as_value().track_exclusive_unbound()
    }
}

impl<'scope, 'data, U: ConstructType> Deref for TypedValue<'scope, 'data, U> {
    type Target = Value<'scope, 'data>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { std::mem::transmute(self) }
    }
}
impl<'scope, 'data, U: ConstructType> DerefMut for TypedValue<'scope, 'data, U> {
    #[inline]
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
    #[inline]
    fn clone(&self) -> Self {
        unsafe { Self::wrap_non_null(self.unwrap_non_null(Private), Private) }
    }
}

impl<T> Copy for TypedValue<'_, '_, T> where T: ConstructType {}

unsafe impl<T> ValidLayout for TypedValue<'_, '_, T>
where
    T: ConstructType,
{
    #[inline]
    fn valid_layout(v: Value) -> bool {
        ValueRef::valid_layout(v)
    }

    #[inline]
    fn type_object<'target, Tgt>(target: &Tgt) -> Value<'target, 'static>
    where
        Tgt: Target<'target>,
    {
        T::base_type(target).expect("Type has no base type")
    }

    const IS_REF: bool = true;
}
unsafe impl<T> ValidField for Option<TypedValue<'_, '_, T>>
where
    T: ConstructType,
{
    #[inline]
    fn valid_field(v: Value) -> bool {
        Option::<ValueRef>::valid_field(v)
    }
}

impl<'scope, 'data, T> ManagedPriv<'scope, 'data> for TypedValue<'scope, 'data, T>
where
    T: ConstructType,
{
    type Wraps = jl_value_t;
    type WithLifetimes<'target, 'da> = TypedValue<'target, 'da, T>;
    const NAME: &'static str = "TypedValue";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    #[inline]
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData, PhantomData, PhantomData)
    }

    #[inline]
    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

unsafe impl<'scope, 'data, T> Typecheck for TypedValue<'scope, 'data, T>
where
    T: ValidLayout + ConstructType,
{
    #[inline]
    fn typecheck(t: DataType) -> bool {
        T::valid_layout(t.as_value())
    }
}

unsafe impl<U> ConstructType for TypedValue<'_, '_, U>
where
    U: ConstructType,
{
    type Static = U::Static;

    #[inline]
    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        U::construct_type_uncached(target)
    }

    #[inline]
    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        U::base_type(target)
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        env: &crate::data::types::construct_type::TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        U::construct_type_with_env_uncached(target, env)
    }
}

use crate::memory::target::TargetType;

pub type TypedValueRef<'scope, 'data, T> = Ref<'scope, 'data, TypedValue<'scope, 'data, T>>;

impl<'scope, 'data> TypedValueRef<'scope, 'data, AnyType> {
    #[inline]
    pub fn from_value_ref(value_ref: ValueRef<'scope, 'data>) -> Self {
        TypedValueRef::wrap(value_ref.ptr())
    }
}

/// A [`TypedValueRef`] with static lifetimes. This is a useful shorthand for signatures of
/// `ccall`able functions that return a [`TypedValue`].
pub type TypedValueRet<T> = Ref<'static, 'static, TypedValue<'static, 'static, T>>;

/// `TypedValue` or `TypedValueRef`, depending on the target type `Tgt`.
pub type TypedValueData<'target, 'data, U, Tgt> =
    <Tgt as TargetType<'target>>::Data<'data, TypedValue<'target, 'data, U>>;

/// `JuliaResult<TypedValue>` or `JuliaResultRef<TypedValueRef>`, depending on the target type
/// `Tgt`.
pub type TypedValueResult<'target, 'data, U, T> =
    TargetResult<'target, 'data, TypedValue<'target, 'data, U>, T>;

pub type TypedValueUnbound<T> = TypedValue<'static, 'static, T>;

unsafe impl<'scope, 'data, T: ConstructType> CCallArg for TypedValue<'scope, 'data, T> {
    type CCallArgType = Value<'scope, 'data>;
    type FunctionArgType = T;
}

unsafe impl<T: ConstructType> CCallReturn for TypedValueRef<'static, 'static, T> {
    type CCallReturnType = Value<'static, 'static>;
    type FunctionReturnType = T;
    type ReturnAs = Self;

    #[inline]
    unsafe fn return_or_throw(self) -> Self::ReturnAs {
        self
    }
}
