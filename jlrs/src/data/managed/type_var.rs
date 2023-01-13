//! Managed type for `TypeVar`.

use std::{marker::PhantomData, ptr::NonNull};

use jl_sys::{jl_new_typevar, jl_tvar_t, jl_tvar_type};
use jlrs_macros::julia_version;

use super::{value::ValueData, Ref};
use crate::{
    convert::to_symbol::ToSymbol,
    data::managed::{
        datatype::DataType,
        private::ManagedPriv,
        symbol::Symbol,
        value::{Value, ValueRef},
        Managed,
    },
    impl_julia_typecheck,
    memory::target::Target,
    private::Private,
};

/// An unknown, but possibly restricted, type parameter. In `Array{T, N}`, `T` and `N` are
/// `TypeVar`s.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct TypeVar<'scope>(NonNull<jl_tvar_t>, PhantomData<&'scope ()>);

impl<'scope> TypeVar<'scope> {
    #[julia_version(windows_lts = false)]
    /// Create a new `TypeVar`, the optional lower and upper bounds must be subtypes of `Type`,
    /// their default values are `Union{}` and `Any` respectively. The returned value can be
    /// cast to a [`TypeVar`]. If Julia throws an exception, it's caught, rooted and returned.
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
        use std::mem::MaybeUninit;

        use crate::catch::catch_exceptions;

        // Safety: if an exception is thrown it's caught and returned
        unsafe {
            let name = name.to_symbol_priv(Private);
            let lb = lower_bound.unwrap_or_else(|| Value::bottom_type(&target));
            let ub = upper_bound.unwrap_or_else(|| DataType::any_type(&target).as_value());

            let mut callback = |result: &mut MaybeUninit<*mut jl_tvar_t>| {
                let res =
                    jl_new_typevar(name.unwrap(Private), lb.unwrap(Private), ub.unwrap(Private));
                result.write(res);

                Ok(())
            };

            let res = match catch_exceptions(&mut callback).unwrap() {
                Ok(tvar) => Ok(NonNull::new_unchecked(tvar)),
                Err(e) => Err(e.ptr()),
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
        let lb = lower_bound.unwrap_or_else(|| Value::bottom_type(&target));
        let ub = upper_bound.unwrap_or_else(|| DataType::any_type(&target).as_value());
        let tvar = jl_new_typevar(name.unwrap(Private), lb.unwrap(Private), ub.unwrap(Private));
        target.data_from_ptr(NonNull::new_unchecked(tvar), Private)
    }

    /*
    inspect(TypeVar):

    name: Symbol (mut)
    lb: Any (mut)
    ub: Any (mut)
    */

    /// The name of this `TypeVar`.
    pub fn name(self) -> Symbol<'scope> {
        // Safety: pointer points to valid data
        unsafe {
            let name = self.unwrap_non_null(Private).as_ref().name;
            debug_assert!(!name.is_null());
            Symbol::wrap_non_null(NonNull::new_unchecked(name), Private)
        }
    }

    /// The lower bound of this `TypeVar`.
    pub fn lower_bound<'target, T>(self, target: T) -> ValueData<'target, 'static, T>
    where
        T: Target<'target>,
    {
        // Safety: pointer points to valid data
        unsafe {
            let lb = self.unwrap_non_null(Private).as_ref().lb;
            debug_assert!(!lb.is_null());
            ValueRef::wrap(NonNull::new_unchecked(lb)).root(target)
        }
    }

    /// The upper bound of this `TypeVar`.
    pub fn upper_bound<'target, T>(self, target: T) -> ValueData<'target, 'static, T>
    where
        T: Target<'target>,
    {
        // Safety: pointer points to valid data
        unsafe {
            let ub = self.unwrap_non_null(Private).as_ref().ub;
            debug_assert!(!ub.is_null());
            ValueRef::wrap(NonNull::new_unchecked(ub)).root(target)
        }
    }
}

impl_julia_typecheck!(TypeVar<'scope>, jl_tvar_type, 'scope);
impl_debug!(TypeVar<'_>);

impl<'scope> ManagedPriv<'scope, '_> for TypeVar<'scope> {
    type Wraps = jl_tvar_t;
    type TypeConstructorPriv<'target, 'da> = TypeVar<'target>;
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

impl_construct_type_managed!(Option<TypeVarRef<'_>>, jl_tvar_type);

/// A reference to a [`TypeVar`] that has not been explicitly rooted.
pub type TypeVarRef<'scope> = Ref<'scope, 'static, TypeVar<'scope>>;

/// A [`TypeVarRef`] with static lifetimes. This is a useful shorthand for signatures of
/// `ccall`able functions that return a [`TypeVar`].
pub type TypeVarRet = Ref<'static, 'static, TypeVar<'static>>;

impl_valid_layout!(TypeVarRef, TypeVar);

use crate::memory::target::target_type::TargetType;

/// `TypeVar` or `TypeVarRef`, depending on the target type `T`.
pub type TypeVarData<'target, T> = <T as TargetType<'target>>::Data<'static, TypeVar<'target>>;

/// `JuliaResult<TypeVar>` or `JuliaResultRef<TypeVarRef>`, depending on the target type `T`.
pub type TypeVarResult<'target, T> = <T as TargetType<'target>>::Result<'static, TypeVar<'target>>;

impl_ccall_arg_managed!(TypeVar, 1);
