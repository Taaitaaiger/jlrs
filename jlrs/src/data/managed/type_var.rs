//! Managed type for `TypeVar`.

use std::{marker::PhantomData, ptr::NonNull};

use jl_sys::{jl_new_typevar, jl_tvar_t, jl_tvar_type, jlrs_tvar_lb, jlrs_tvar_name, jlrs_tvar_ub};

use super::{value::ValueData, Ref};
use crate::{
    catch::{catch_exceptions, unwrap_exc},
    convert::to_symbol::ToSymbol,
    data::managed::{
        datatype::DataType,
        private::ManagedPriv,
        symbol::Symbol,
        value::{Value, ValueRef},
        Managed,
    },
    impl_julia_typecheck,
    memory::target::{unrooted::Unrooted, Target, TargetResult},
    private::Private,
};

/// An unknown, but possibly restricted, type parameter. In `Array{T, N}`, `T` and `N` are
/// `TypeVar`s.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct TypeVar<'scope>(NonNull<jl_tvar_t>, PhantomData<&'scope ()>);

impl<'scope> TypeVar<'scope> {
    /// Create a new `TypeVar`, the optional lower and upper bounds must be subtypes of `Type`,
    /// their default values are `Union{}` and `Any` respectively. The returned value can be
    /// cast to a [`TypeVar`]. If Julia throws an exception, it's caught, rooted and returned.
    pub fn new<'target, N, Tgt>(
        target: Tgt,
        name: N,
        lower_bound: Option<Value>,
        upper_bound: Option<Value>,
    ) -> TypeVarResult<'target, Tgt>
    where
        Tgt: Target<'target>,
        N: ToSymbol,
    {
        // Safety: if an exception is thrown it's caught and returned
        unsafe {
            let name = name.to_symbol_priv(Private);
            let lb = lower_bound.unwrap_or_else(|| Value::bottom_type(&target));
            let ub = upper_bound.unwrap_or_else(|| DataType::any_type(&target).as_value());

            let callback =
                || jl_new_typevar(name.unwrap(Private), lb.unwrap(Private), ub.unwrap(Private));

            let res = match catch_exceptions(callback, unwrap_exc) {
                Ok(tvar) => Ok(NonNull::new_unchecked(tvar)),
                Err(e) => Err(e),
            };

            target.result_from_ptr(res, Private)
        }
    }

    /// Create a new `TypeVar`, the optional lower and upper bounds must be subtypes of `Type`,
    /// their default values are `Union{}` and `Any` respectively. The returned value can be
    /// cast to a [`TypeVar`]. If Julia throws an exception it isn't caught.
    ///
    /// Safety: an exception must not be thrown if this method is called from a `ccall`ed
    /// function.
    #[inline]
    pub unsafe fn new_unchecked<'target, N, Tgt>(
        target: Tgt,
        name: N,
        lower_bound: Option<Value>,
        upper_bound: Option<Value>,
    ) -> TypeVarData<'target, Tgt>
    where
        Tgt: Target<'target>,
        N: ToSymbol,
    {
        let name = name.to_symbol_priv(Private);
        let lb = lower_bound.unwrap_or_else(|| Value::bottom_type(&target));
        let ub = upper_bound.unwrap_or_else(|| DataType::any_type(&target).as_value());
        let tvar = jl_new_typevar(name.unwrap(Private), lb.unwrap(Private), ub.unwrap(Private));
        target.data_from_ptr(NonNull::new_unchecked(tvar), Private)
    }

    /// Returns `true` if one of the bounds has an indirect type parameter.
    pub fn has_indirect_typevar(self, tvar: TypeVar) -> bool {
        unsafe {
            let unrooted = Unrooted::new();

            let ub = self.upper_bound(unrooted).as_value();
            if ub.has_typevar(tvar) {
                return true;
            }
            if ub.is::<DataType>() {
                let ub = ub.cast_unchecked::<DataType>();
                if ub.has_indirect_typevar(tvar) {
                    return true;
                }
            }

            let lb = self.lower_bound(unrooted).as_value();
            if lb.has_typevar(tvar) {
                return true;
            }
            if lb.is::<DataType>() {
                let lb = lb.cast_unchecked::<DataType>();
                if lb.has_indirect_typevar(tvar) {
                    return true;
                }
            }
        }

        false
    }

    /// The name of this `TypeVar`.
    #[inline]
    pub fn name(self) -> Symbol<'scope> {
        // Safety: pointer points to valid data
        unsafe {
            let name = jlrs_tvar_name(self.unwrap(Private));
            debug_assert!(!name.is_null());
            Symbol::wrap_non_null(NonNull::new_unchecked(name), Private)
        }
    }

    /// The lower bound of this `TypeVar`.
    #[inline]
    pub fn lower_bound<'target, Tgt>(self, target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        // Safety: pointer points to valid data
        unsafe {
            let lb = jlrs_tvar_lb(self.unwrap(Private));
            debug_assert!(!lb.is_null());
            ValueRef::wrap(NonNull::new_unchecked(lb)).root(target)
        }
    }

    /// The upper bound of this `TypeVar`.
    #[inline]
    pub fn upper_bound<'target, Tgt>(self, target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        // Safety: pointer points to valid data
        unsafe {
            let ub = jlrs_tvar_ub(self.unwrap(Private));
            debug_assert!(!ub.is_null());
            ValueRef::wrap(NonNull::new_unchecked(ub)).root(target)
        }
    }
}

impl_julia_typecheck!(TypeVar<'scope>, jl_tvar_type, 'scope);
impl_debug!(TypeVar<'_>);

impl<'scope> ManagedPriv<'scope, '_> for TypeVar<'scope> {
    type Wraps = jl_tvar_t;
    type WithLifetimes<'target, 'da> = TypeVar<'target>;
    const NAME: &'static str = "TypeVar";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    #[inline]
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    #[inline]
    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

impl_construct_type_managed!(TypeVar, 1, jl_tvar_type);

/// A reference to a [`TypeVar`] that has not been explicitly rooted.
pub type TypeVarRef<'scope> = Ref<'scope, 'static, TypeVar<'scope>>;

/// A [`TypeVarRef`] with static lifetimes. This is a useful shorthand for signatures of
/// `ccall`able functions that return a [`TypeVar`].
pub type TypeVarRet = Ref<'static, 'static, TypeVar<'static>>;

impl_valid_layout!(TypeVarRef, TypeVar, jl_tvar_type);

use crate::memory::target::TargetType;

/// `TypeVar` or `TypeVarRef`, depending on the target type `Tgt`.
pub type TypeVarData<'target, Tgt> = <Tgt as TargetType<'target>>::Data<'static, TypeVar<'target>>;

/// `JuliaResult<TypeVar>` or `JuliaResultRef<TypeVarRef>`, depending on the target type `Tgt`.
pub type TypeVarResult<'target, Tgt> = TargetResult<'target, 'static, TypeVar<'target>, Tgt>;

impl_ccall_arg_managed!(TypeVar, 1);
impl_into_typed!(TypeVar);
