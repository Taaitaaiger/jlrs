//! Support for values with the `Core.TypeVar` type.

use super::symbol::Symbol;
use super::union_all::UnionAll;
use super::Value;
use crate::global::Global;
use crate::traits::private::Internal;
use crate::{
    error::{JlrsError, JlrsResult},
    traits::{cast::Cast, frame::Frame, temporary_symbol::TemporarySymbol},
};
use crate::{impl_julia_type, impl_julia_typecheck, impl_valid_layout};
use jl_sys::{jl_any_type, jl_bottom_type, jl_new_typevar, jl_tvar_t, jl_tvar_type};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
};

/// This is a unknown, but possibly restricted, type parameter. In `Array{T, N}`, `T` and `N` are
/// `TypeVar`s.
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct TypeVar<'frame>(*mut jl_tvar_t, PhantomData<&'frame ()>);

impl<'frame> TypeVar<'frame> {
    pub(crate) unsafe fn wrap(type_var: *mut jl_tvar_t) -> Self {
        TypeVar(type_var, PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn ptr(self) -> *mut jl_tvar_t {
        self.0
    }

    /// Create a new `TypeVar`, the optional lower and upper bounds must be subtypes of `Type`,
    /// their default values are `Union{}` and `Any` respectively. This requires one slot on the
    /// GC stack.
    pub fn new<F, S>(
        frame: &mut F,
        name: S,
        lower_bound: Option<Value>,
        upper_bound: Option<Value>,
    ) -> JlrsResult<Self>
    where
        F: Frame<'frame>,
        S: TemporarySymbol,
    {
        unsafe {
            let global = Global::new();
            let name = name.temporary_symbol(Internal);

            let lb = lower_bound.map_or(jl_bottom_type.cast(), |v| v.ptr());
            if !Value::wrap(lb)
                .datatype()
                .unwrap()
                .as_value()
                .subtype(UnionAll::type_type(global).as_value())
            {
                Err(JlrsError::NotATypeLB(name.as_string()))?;
            }

            let ub = upper_bound.map_or(jl_any_type.cast(), |v| v.ptr());
            if !Value::wrap(ub)
                .datatype()
                .unwrap()
                .as_value()
                .subtype(UnionAll::type_type(global).as_value())
            {
                Err(JlrsError::NotATypeUB(name.as_string()))?;
            }

            let tvar = jl_new_typevar(name.ptr(), lb, ub);
            frame
                .protect(tvar.cast(), Internal)
                .map_err(JlrsError::alloc_error)?;

            Ok(Self::wrap(tvar))
        }
    }

    /// The name of this `TypeVar`.
    pub fn name(self) -> Symbol<'frame> {
        unsafe { Symbol::wrap((&*self.ptr()).name) }
    }

    /// The lower bound of this `TypeVar`.
    pub fn lower_bound(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap((&*self.ptr()).lb) }
    }

    /// The upper bound of this `TypeVar`.
    pub fn upper_bound(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap((&*self.ptr()).ub) }
    }

    /// Convert `self` to a `Value`.
    pub fn as_value(self) -> Value<'frame, 'static> {
        self.into()
    }
}

impl<'scope> Debug for TypeVar<'scope> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("TypeVar")
            .field(&self.name().as_string())
            .finish()
    }
}

impl<'frame> Into<Value<'frame, 'static>> for TypeVar<'frame> {
    fn into(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap(self.ptr().cast()) }
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for TypeVar<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotATypeVar)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        Self::wrap(value.ptr().cast())
    }
}

impl_julia_typecheck!(TypeVar<'frame>, jl_tvar_type, 'frame);
impl_julia_type!(TypeVar<'frame>, jl_tvar_type, 'frame);
impl_valid_layout!(TypeVar<'frame>, 'frame);
