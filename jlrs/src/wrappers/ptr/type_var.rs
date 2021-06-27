//! Wrapper for `TypeVar`.

use super::TypeVarRef;
use super::{datatype::DataType, value::Value};
use super::{private::Wrapper, Wrapper as _};
use crate::memory::global::Global;
use crate::memory::scope::Scope;
use crate::private::Private;
use crate::wrappers::ptr::{SymbolRef, ValueRef};
use crate::{
    convert::temporary_symbol::TemporarySymbol,
    error::{JlrsError, JlrsResult},
    memory::frame::Frame,
};
use crate::{impl_debug, impl_julia_typecheck, impl_valid_layout};
use jl_sys::{jl_new_typevar, jl_tvar_t, jl_tvar_type};
use std::{marker::PhantomData, ptr::NonNull};

/// An unknown, but possibly restricted, type parameter. In `Array{T, N}`, `T` and `N` are
/// `TypeVar`s.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct TypeVar<'scope>(NonNull<jl_tvar_t>, PhantomData<&'scope ()>);

impl<'scope> TypeVar<'scope> {
    /// Create a new `TypeVar`, the optional lower and upper bounds must be subtypes of `Type`,
    /// their default values are `Union{}` and `Any` respectively. The returned value can be
    /// cast to a [`TypeVar`].
    pub fn new<'target, 'current, N, S, F>(
        scope: S,
        name: N,
        lower_bound: Option<Value>,
        upper_bound: Option<Value>,
    ) -> JlrsResult<S::Value>
    where
        S: Scope<'target, 'current, 'static, F>,
        F: Frame<'current>,
        N: TemporarySymbol,
    {
        unsafe {
            let global = Global::new();
            let name = name.temporary_symbol(Private);
            let bottom = Value::bottom_type(global);

            let lb = lower_bound.unwrap_or(bottom);

            if lb != bottom && !lb.is_type() && !lb.is::<TypeVar>() {
                Err(JlrsError::NotATypeLB {
                    typevar_name: name.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                })?;
            }

            let upper = DataType::any_type(global).as_value();
            let ub = upper_bound.unwrap_or(upper);

            if ub != upper && !ub.is_type() && !ub.is::<TypeVar>() {
                Err(JlrsError::NotATypeUB {
                    typevar_name: name.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                })?;
            }

            let tvar = jl_new_typevar(name.unwrap(Private), lb.unwrap(Private), ub.unwrap(Private));

            scope.value(NonNull::new_unchecked(tvar.cast()), Private)
        }
    }

    /// Create a new `TypeVar`, the optional lower and upper bounds must be subtypes of `Type`,
    /// their default values are `Union{}` and `Any` respectively. Unlike [`TypeVar::new`], this
    /// method doesn't root the allocated value.
    pub fn new_unrooted<'global, N, S, F>(
        global: Global<'global>,
        name: N,
        lower_bound: Option<Value>,
        upper_bound: Option<Value>,
    ) -> JlrsResult<TypeVarRef<'scope>>
    where
        N: TemporarySymbol,
    {
        unsafe {
            let name = name.temporary_symbol(Private);
            let bottom = Value::bottom_type(global);

            let lb = lower_bound.unwrap_or(bottom);

            if lb != bottom && !lb.is_type() && !lb.is::<TypeVar>() {
                Err(JlrsError::NotATypeLB {
                    typevar_name: name.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                })?;
            }

            let upper = DataType::any_type(global).as_value();
            let ub = upper_bound.unwrap_or(upper);

            if ub != upper && !ub.is_type() && !ub.is::<TypeVar>() {
                Err(JlrsError::NotATypeUB {
                    typevar_name: name.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                })?;
            }

            let tvar = jl_new_typevar(name.unwrap(Private), lb.unwrap(Private), ub.unwrap(Private));

            Ok(TypeVarRef::wrap(tvar))
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
    pub fn name(self) -> SymbolRef<'scope> {
        unsafe { SymbolRef::wrap(self.unwrap_non_null(Private).as_ref().name) }
    }

    /// The lower bound of this `TypeVar`.
    pub fn lower_bound(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().lb) }
    }

    /// The upper bound of this `TypeVar`.
    pub fn upper_bound(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().ub) }
    }
}

impl_julia_typecheck!(TypeVar<'scope>, jl_tvar_type, 'scope);
impl_debug!(TypeVar<'_>);
impl_valid_layout!(TypeVar<'scope>, 'scope);

impl<'scope> Wrapper<'scope, '_> for TypeVar<'scope> {
    type Internal = jl_tvar_t;
    const NAME: &'static str = "TypeVar";

    unsafe fn wrap_non_null(inner: NonNull<Self::Internal>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    unsafe fn unwrap_non_null(self, _: Private) -> NonNull<Self::Internal> {
        self.0
    }
}
