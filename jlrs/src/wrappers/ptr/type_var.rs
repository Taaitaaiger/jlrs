//! Wrapper for `TypeVar`.

use crate::{
    convert::to_symbol::ToSymbol,
    error::JlrsResult,
    impl_debug, impl_julia_typecheck,
    memory::{global::Global, output::Output, scope::PartialScope},
    private::Private,
    wrappers::ptr::{
        datatype::DataType, private::WrapperPriv, symbol::SymbolRef, value::Value, value::ValueRef,
        Wrapper,
    },
};
use cfg_if::cfg_if;
use jl_sys::{jl_new_typevar, jl_tvar_t, jl_tvar_type};
use std::{marker::PhantomData, ptr::NonNull};

use super::Ref;

cfg_if! {
    if #[cfg(not(all(target_os = "windows", feature = "lts")))] {
        use crate::error::{JuliaResult, JuliaResultRef};
        use jl_sys::{jlrs_new_typevar, jlrs_result_tag_t_JLRS_RESULT_ERR};
    }
}

/// An unknown, but possibly restricted, type parameter. In `Array{T, N}`, `T` and `N` are
/// `TypeVar`s.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct TypeVar<'scope>(NonNull<jl_tvar_t>, PhantomData<&'scope ()>);

impl<'scope> TypeVar<'scope> {
    /// Create a new `TypeVar`, the optional lower and upper bounds must be subtypes of `Type`,
    /// their default values are `Union{}` and `Any` respectively. The returned value can be
    /// cast to a [`TypeVar`]. If Julia throws an exception, it's caught, rooted and returned.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub fn new<'target, N, S>(
        scope: S,
        name: N,
        lower_bound: Option<Value>,
        upper_bound: Option<Value>,
    ) -> JlrsResult<JuliaResult<'target, 'static, TypeVar<'target>>>
    where
        S: PartialScope<'target>,
        N: ToSymbol,
    {
        let global = scope.global();

        // Safety: the result is rooted immediately. If an exception is thrown it's caught and returned.
        unsafe {
            let v = match Self::new_unrooted(global, name, lower_bound, upper_bound) {
                Ok(v) => Ok(v.root(scope)?),
                Err(e) => Err(e.root(scope)?),
            };

            Ok(v)
        }
    }

    /// Create a new `TypeVar`, the optional lower and upper bounds must be subtypes of `Type`,
    /// their default values are `Union{}` and `Any` respectively. The returned value can be
    /// cast to a [`TypeVar`]. If Julia throws an exception it isn't caught.
    ///
    /// Safety: an exception must not be thrown if this method is called from a `ccall`ed
    /// function.
    pub unsafe fn new_unchecked<'target, N, S>(
        scope: S,
        name: N,
        lower_bound: Option<Value>,
        upper_bound: Option<Value>,
    ) -> JlrsResult<TypeVar<'target>>
    where
        S: PartialScope<'target>,
        N: ToSymbol,
    {
        let global = scope.global();
        Self::new_unrooted_unchecked(global, name, lower_bound, upper_bound).root(scope)
    }

    /// See [`TypeVar::new`], the only difference is that the result isn't rooted.
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    pub fn new_unrooted<'global, N>(
        global: Global<'global>,
        name: N,
        lower_bound: Option<Value>,
        upper_bound: Option<Value>,
    ) -> JuliaResultRef<'global, 'static, TypeVarRef<'global>>
    where
        N: ToSymbol,
    {
        // Safety: if an exception is thrown it's caught and returned
        unsafe {
            let name = name.to_symbol_priv(Private);
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
    ///
    /// Safety: an exception must not be thrown if this method is called from a `ccall`ed
    /// function.
    pub unsafe fn new_unrooted_unchecked<'global, N>(
        global: Global<'global>,
        name: N,
        lower_bound: Option<Value>,
        upper_bound: Option<Value>,
    ) -> TypeVarRef<'scope>
    where
        N: ToSymbol,
    {
        let name = name.to_symbol_priv(Private);
        let lb = lower_bound.unwrap_or_else(|| Value::bottom_type(global));
        let ub = upper_bound.unwrap_or_else(|| DataType::any_type(global).as_value());
        let tvar = jl_new_typevar(name.unwrap(Private), lb.unwrap(Private), ub.unwrap(Private));

        TypeVarRef::wrap(tvar)
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
        // Safety: pointer points to valid data
        unsafe { SymbolRef::wrap(self.unwrap_non_null(Private).as_ref().name) }
    }

    /// The lower bound of this `TypeVar`.
    pub fn lower_bound(self) -> ValueRef<'scope, 'static> {
        // Safety: pointer points to valid data
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().lb) }
    }

    /// The upper bound of this `TypeVar`.
    pub fn upper_bound(self) -> ValueRef<'scope, 'static> {
        // Safety: pointer points to valid data
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().ub) }
    }

    /// Use the `Output` to extend the lifetime of this data.
    pub fn root<'target>(self, output: Output<'target>) -> TypeVar<'target> {
        // Safety: pointer points to valid data
        unsafe {
            let ptr = self.unwrap_non_null(Private);
            output.set_root::<TypeVar>(ptr);
            TypeVar::wrap_non_null(ptr, Private)
        }
    }
}

impl_julia_typecheck!(TypeVar<'scope>, jl_tvar_type, 'scope);
impl_debug!(TypeVar<'_>);

impl<'scope> WrapperPriv<'scope, '_> for TypeVar<'scope> {
    type Wraps = jl_tvar_t;
    const NAME: &'static str = "TypeVar";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

impl_root!(TypeVar, 1);

/// A reference to a [`TypeVar`] that has not been explicitly rooted.
pub type TypeVarRef<'scope> = Ref<'scope, 'static, TypeVar<'scope>>;
impl_valid_layout!(TypeVarRef, TypeVar);
impl_ref_root!(TypeVar, TypeVarRef, 1);
