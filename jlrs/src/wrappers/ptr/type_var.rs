//! Wrapper for `TypeVar`.

use crate::{
    convert::to_symbol::ToSymbol,
    impl_julia_typecheck,
    memory::target::Target,
    private::Private,
    wrappers::ptr::{
        datatype::DataType, private::WrapperPriv, symbol::SymbolRef, value::Value, value::ValueRef,
        Wrapper,
    },
};
use jl_sys::{jl_new_typevar, jl_tvar_t, jl_tvar_type};
use std::{marker::PhantomData, ptr::NonNull};

use super::Ref;

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
    pub fn new<'target, N, T>(
        target: T,
        name: N,
        lower_bound: Option<Value>,
        upper_bound: Option<Value>,
    ) -> TypeVarResult<'target, T>
    where
        T: Target<'target>,
        N: ToSymbol,
    {
        use crate::catch::catch_exceptions;
        use std::mem::MaybeUninit;

        // Safety: if an exception is thrown it's caught and returned
        unsafe {
            let name = name.to_symbol_priv(Private);
            let global = target.global();
            let lb = lower_bound.unwrap_or_else(|| Value::bottom_type(&global));
            let ub = upper_bound.unwrap_or_else(|| DataType::any_type(&global).as_value());

            let mut callback = |result: &mut MaybeUninit<*mut jl_tvar_t>| {
                let res =
                    jl_new_typevar(name.unwrap(Private), lb.unwrap(Private), ub.unwrap(Private));
                result.write(res);

                Ok(())
            };

            let res = match catch_exceptions(&mut callback).unwrap() {
                Ok(tvar) => Ok(NonNull::new_unchecked(tvar)),
                Err(e) => Err(NonNull::new_unchecked(e.ptr())),
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
    pub unsafe fn new_unchecked<'target, N, T>(
        target: T,
        name: N,
        lower_bound: Option<Value>,
        upper_bound: Option<Value>,
    ) -> TypeVarData<'target, T>
    where
        T: Target<'target>,
        N: ToSymbol,
    {
        let name = name.to_symbol_priv(Private);
        let global = target.global();
        let lb = lower_bound.unwrap_or_else(|| Value::bottom_type(&global));
        let ub = upper_bound.unwrap_or_else(|| DataType::any_type(&global).as_value());
        let tvar = jl_new_typevar(name.unwrap(Private), lb.unwrap(Private), ub.unwrap(Private));
        target.data_from_ptr(NonNull::new_unchecked(tvar), Private)
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

    /// Use the target to reroot this data.
    pub fn root<'target, T>(self, target: T) -> TypeVarData<'target, T>
    where
        T: Target<'target>,
    {
        // Safety: the data is valid.
        unsafe { target.data_from_ptr(self.unwrap_non_null(Private), Private) }
    }
}

impl_julia_typecheck!(TypeVar<'scope>, jl_tvar_type, 'scope);
impl_debug!(TypeVar<'_>);

impl<'scope> WrapperPriv<'scope, '_> for TypeVar<'scope> {
    type Wraps = jl_tvar_t;
    type StaticPriv = TypeVar<'static>;
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

use crate::memory::target::target_type::TargetType;
pub type TypeVarData<'target, T> = <T as TargetType<'target>>::Data<'static, TypeVar<'target>>;
pub type TypeVarResult<'target, T> = <T as TargetType<'target>>::Result<'static, TypeVar<'target>>;
