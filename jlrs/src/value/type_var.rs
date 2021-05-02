//! Support for values with the `Core.TypeVar` type.

use super::union_all::UnionAll;
use super::wrapper_ref::{SymbolRef, ValueRef};
use super::Value;
use crate::memory::global::Global;
use crate::private::Private;
use crate::{
    convert::{cast::Cast, temporary_symbol::TemporarySymbol},
    error::{JlrsError, JlrsResult},
    memory::traits::frame::Frame,
};
use crate::{impl_julia_type, impl_julia_typecheck, impl_valid_layout};
use jl_sys::{jl_any_type, jl_bottom_type, jl_new_typevar, jl_tvar_t, jl_tvar_type};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    ptr::NonNull,
};

/// This is a unknown, but possibly restricted, type parameter. In `Array{T, N}`, `T` and `N` are
/// `TypeVar`s.
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct TypeVar<'frame>(NonNull<jl_tvar_t>, PhantomData<&'frame ()>);

impl<'frame> TypeVar<'frame> {
    pub(crate) unsafe fn wrap(type_var: *mut jl_tvar_t) -> Self {
        debug_assert!(!type_var.is_null());
        TypeVar(NonNull::new_unchecked(type_var), PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn inner(self) -> NonNull<jl_tvar_t> {
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
            let name = name.temporary_symbol(Private);

            let lb = lower_bound.map_or(jl_bottom_type.cast(), |v| v.inner().as_ptr());
            if !Value::wrap(lb)
                .datatype()
                .as_value()
                .subtype(UnionAll::type_type(global).as_value())
            {
                Err(JlrsError::NotATypeLB(
                    name.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                ))?;
            }

            let ub = upper_bound.map_or(jl_any_type.cast(), |v| v.inner().as_ptr());
            if !Value::wrap(ub)
                .datatype()
                .as_value()
                .subtype(UnionAll::type_type(global).as_value())
            {
                Err(JlrsError::NotATypeUB(
                    name.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                ))?;
            }

            let tvar = jl_new_typevar(name.inner().as_ptr(), lb, ub);
            frame
                .push_root(tvar.cast(), Private)
                .map_err(JlrsError::alloc_error)?;

            Ok(Self::wrap(tvar))
        }
    }

    /*
    for (a, b) in zip(fieldnames(TypeVar), fieldtypes(TypeVar))
        println(a, ": ", b)
    end
    name: Symbol
    lb: Any
    ub: Any
    */

    /// The name of this `TypeVar`.
    pub fn name(self) -> SymbolRef<'frame> {
        unsafe { SymbolRef::wrap((&*self.inner().as_ptr()).name) }
    }

    /// The lower bound of this `TypeVar`.
    pub fn lower_bound(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap((&*self.inner().as_ptr()).lb) }
    }

    /// The upper bound of this `TypeVar`.
    pub fn upper_bound(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap((&*self.inner().as_ptr()).ub) }
    }

    /// Convert `self` to a `Value`.
    pub fn as_value(self) -> Value<'frame, 'static> {
        self.into()
    }
}

impl<'scope> Debug for TypeVar<'scope> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        unsafe {
            f.debug_tuple("TypeVar")
                .field(&self.name().assume_reachable_unchecked().as_string())
                .finish()
        }
    }
}

impl<'frame> Into<Value<'frame, 'static>> for TypeVar<'frame> {
    fn into(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap(self.inner().as_ptr().cast()) }
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
        Self::wrap(value.inner().as_ptr().cast())
    }
}

impl_julia_typecheck!(TypeVar<'frame>, jl_tvar_type, 'frame);
impl_julia_type!(TypeVar<'frame>, jl_tvar_type, 'frame);
impl_valid_layout!(TypeVar<'frame>, 'frame);
