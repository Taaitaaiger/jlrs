//! Wrapper for `TypeVar`.

use super::TypeVarRef;
use super::{datatype::DataType, value::Value};
use super::{private::Wrapper, Wrapper as _};
use crate::error::JuliaResultRef;
use crate::memory::global::Global;
use crate::memory::scope::Scope;
use crate::private::Private;
use crate::wrappers::ptr::{SymbolRef, ValueRef};
use crate::{convert::temporary_symbol::TemporarySymbol, error::JlrsResult, memory::frame::Frame};
use crate::{impl_debug, impl_julia_typecheck, impl_valid_layout};
use jl_sys::{
    jl_new_typevar, jl_tvar_t, jl_tvar_type, jlrs_new_typevar, jlrs_result_tag_t_JLRS_RESULT_ERR,
};
use std::{marker::PhantomData, ptr::NonNull};

/// An unknown, but possibly restricted, type parameter. In `Array{T, N}`, `T` and `N` are
/// `TypeVar`s.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct TypeVar<'scope>(NonNull<jl_tvar_t>, PhantomData<&'scope ()>);

impl<'scope> TypeVar<'scope> {
    /// Create a new `TypeVar`, the optional lower and upper bounds must be subtypes of `Type`,
    /// their default values are `Union{}` and `Any` respectively. The returned value can be
    /// cast to a [`TypeVar`]. If Julia throws an exception, it's caught, rooted and returned.
    pub fn new<'target, 'current, N, S, F>(
        scope: S,
        name: N,
        lower_bound: Option<Value>,
        upper_bound: Option<Value>,
    ) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'static, F>,
        F: Frame<'current>,
        N: TemporarySymbol,
    {
        unsafe {
            let global = scope.global();
            let name = name.temporary_symbol(Private);
            let lb = lower_bound.unwrap_or_else(|| Value::bottom_type(global));
            let ub = upper_bound.unwrap_or_else(|| DataType::any_type(global).as_value());
            let tvar =
                jlrs_new_typevar(name.unwrap(Private), lb.unwrap(Private), ub.unwrap(Private));

            if tvar.flag == jlrs_result_tag_t_JLRS_RESULT_ERR {
                scope.call_result(Err(NonNull::new_unchecked(tvar.data)), Private)
            } else {
                scope.call_result(Ok(NonNull::new_unchecked(tvar.data).cast()), Private)
            }
        }
    }

    /// Create a new `TypeVar`, the optional lower and upper bounds must be subtypes of `Type`,
    /// their default values are `Union{}` and `Any` respectively. The returned value can be
    /// cast to a [`TypeVar`]. If Julia throws an exception the process aborts.
    pub fn new_unchecked<'target, 'current, N, S, F>(
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
            let global = scope.global();
            let name = name.temporary_symbol(Private);
            let lb = lower_bound.unwrap_or_else(|| Value::bottom_type(global));
            let ub = upper_bound.unwrap_or_else(|| DataType::any_type(global).as_value());
            let tvar = jl_new_typevar(name.unwrap(Private), lb.unwrap(Private), ub.unwrap(Private));

            scope.value(NonNull::new_unchecked(tvar.cast()), Private)
        }
    }

    /// See [`TypeVar::new`], the only difference is that the result isn't rooted.
    pub fn new_unrooted<'global, N>(
        global: Global<'global>,
        name: N,
        lower_bound: Option<Value>,
        upper_bound: Option<Value>,
    ) -> JuliaResultRef<'global, 'static, TypeVarRef<'global>>
    where
        N: TemporarySymbol,
    {
        unsafe {
            let name = name.temporary_symbol(Private);
            let lb = lower_bound.unwrap_or_else(|| Value::bottom_type(global));
            let ub = upper_bound.unwrap_or_else(|| DataType::any_type(global).as_value());
            let tvar =
                jlrs_new_typevar(name.unwrap(Private), lb.unwrap(Private), ub.unwrap(Private));
            if tvar.flag == jlrs_result_tag_t_JLRS_RESULT_ERR {
                Err(ValueRef::wrap(tvar.data))
            } else {
                Ok(TypeVarRef::wrap(tvar.data.cast()))
            }
        }
    }

    /// See [`TypeVar::new_unchecked`], the only difference is that the result isn't rooted.
    pub fn new_unrooted_unchecked<'global, N, S, F>(
        global: Global<'global>,
        name: N,
        lower_bound: Option<Value>,
        upper_bound: Option<Value>,
    ) -> TypeVarRef<'scope>
    where
        N: TemporarySymbol,
    {
        unsafe {
            let name = name.temporary_symbol(Private);
            let lb = lower_bound.unwrap_or_else(|| Value::bottom_type(global));
            let ub = upper_bound.unwrap_or_else(|| DataType::any_type(global).as_value());
            let tvar = jl_new_typevar(name.unwrap(Private), lb.unwrap(Private), ub.unwrap(Private));

            TypeVarRef::wrap(tvar)
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
