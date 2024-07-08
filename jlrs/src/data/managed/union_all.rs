//! Managed type for `UnionAll`, A union of types over all values of a type parameter.

use std::{marker::PhantomData, ptr::NonNull};

#[julia_version(until = "1.6")]
use jl_sys::jl_vararg_type;
use jl_sys::{
    jl_abstractarray_type, jl_anytuple_type_type, jl_apply_type, jl_array_type, jl_densearray_type,
    jl_llvmpointer_type, jl_namedtuple_type, jl_pointer_type, jl_ref_type, jl_type_type,
    jl_type_unionall, jl_unionall_t, jl_unionall_type, jl_value_t, jlrs_unionall_body,
    jlrs_unionall_tvar,
};
use jlrs_macros::julia_version;

use super::{
    erase_scope_lifetime,
    value::{ValueData, ValueResult},
    Managed, Ref,
};
use crate::{
    catch::{catch_exceptions, unwrap_exc},
    data::managed::{datatype::DataType, private::ManagedPriv, type_var::TypeVar, value::Value},
    impl_julia_typecheck,
    memory::target::{Target, TargetResult},
    private::Private,
};

/// An iterated union of types. If a struct field has a parametric type with some of its
/// parameters unknown, its type is represented by a `UnionAll`.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct UnionAll<'scope>(NonNull<jl_unionall_t>, PhantomData<&'scope ()>);

impl<'scope> UnionAll<'scope> {
    /// Create a new `UnionAll`. If an exception is thrown, it's caught and returned.
    pub fn new<'target, Tgt>(
        target: Tgt,
        tvar: TypeVar,
        body: Value<'_, 'static>,
    ) -> ValueResult<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        // Safety: if an exception is thrown it's caught, the result is immediately rooted
        unsafe {
            let callback = || jl_type_unionall(tvar.unwrap(Private), body.unwrap(Private));

            let res = match catch_exceptions(callback, unwrap_exc) {
                Ok(ptr) => Ok(NonNull::new_unchecked(ptr)),
                Err(e) => Err(e),
            };

            target.result_from_ptr(res, Private)
        }
    }

    /// Create a new `UnionAll`. If an exception is thrown it isn't caught
    ///
    /// Safety: an exception must not be thrown if this method is called from a `ccall`ed
    /// function.
    #[inline]
    pub unsafe fn new_unchecked<'target, Tgt>(
        target: Tgt,
        tvar: TypeVar,
        body: Value<'_, 'static>,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        let ua = jl_type_unionall(tvar.unwrap(Private), body.unwrap(Private));
        target.data_from_ptr(NonNull::new_unchecked(ua), Private)
    }

    /// The type at the bottom of this `UnionAll`.
    #[inline]
    pub fn base_type(self) -> DataType<'scope> {
        let mut b = self;

        unsafe {
            // Safety: pointer points to valid data
            while b.body().is::<UnionAll>() {
                b = b.body().cast_unchecked();
            }

            // Safety: type at the base must be a DataType
            b.body().cast_unchecked::<DataType>()
        }
    }

    /// The body of this `UnionAll`. This is either another `UnionAll` or a `DataType`.
    #[inline]
    pub fn body(self) -> Value<'scope, 'static> {
        // Safety: pointer points to valid data
        unsafe {
            let body = jlrs_unionall_body(self.unwrap(Private));
            debug_assert!(!body.is_null());
            Value::wrap_non_null(NonNull::new_unchecked(body), Private)
        }
    }

    /// The type variable associated with this "layer" of the `UnionAll`.
    #[inline]
    pub fn var(self) -> TypeVar<'scope> {
        // Safety: pointer points to valid data
        unsafe {
            let var = jlrs_unionall_tvar(self.unwrap(Private));
            debug_assert!(!var.is_null());
            TypeVar::wrap_non_null(NonNull::new_unchecked(var), Private)
        }
    }

    /// Apply `types` to this `UnionAll`.
    ///
    /// If the result has free type parameters, it's returned as a `DataType` with free type
    /// parameters. Call `UnionAll::rewrap` to turn such a type into a `UnionAll`.
    pub unsafe fn apply_types<'target, 'params, V, Tgt>(
        self,
        target: Tgt,
        types: V,
    ) -> ValueResult<'target, 'static, Tgt>
    where
        V: AsRef<[Value<'params, 'static>]>,
        Tgt: Target<'target>,
    {
        let types = types.as_ref();
        let n = types.len();
        let types_ptr = types.as_ptr() as *mut *mut jl_value_t;
        unsafe {
            let callback = || jl_apply_type(self.as_value().unwrap(Private), types_ptr, n);

            let res = match catch_exceptions(callback, unwrap_exc) {
                Ok(ptr) => Ok(NonNull::new_unchecked(ptr)),
                Err(e) => Err(e),
            };

            target.result_from_ptr(res, Private)
        }
    }

    /// Apply `types` to this `UnionAll` without catching exceptions.
    ///
    /// If the result has free type parameters, it's returned as a `DataType` with free type
    /// parameters. Call `UnionAll::rewrap` to turn such a type into a `UnionAll`.
    ///
    /// Safety: if an exception is throw it isn't caught.
    #[inline]
    pub unsafe fn apply_types_unchecked<'target, 'params, V, Tgt>(
        self,
        target: Tgt,
        types: V,
    ) -> ValueData<'target, 'static, Tgt>
    where
        V: AsRef<[Value<'params, 'static>]>,
        Tgt: Target<'target>,
    {
        let types = types.as_ref();
        let n = types.len();
        let types_ptr = types.as_ptr() as *mut *mut jl_value_t;
        let applied = jl_apply_type(self.as_value().unwrap(Private), types_ptr, n);
        debug_assert!(!applied.is_null());
        target.data_from_ptr(NonNull::new_unchecked(applied), Private)
    }

    /// Wrap `ty` with its free type parameters.
    pub fn rewrap<'target, Tgt: Target<'target>>(
        target: Tgt,
        ty: DataType,
    ) -> ValueData<'target, 'static, Tgt> {
        target.with_local_scope::<_, _, 1>(|target, mut frame| unsafe {
            let params = ty.parameters();
            let params = params.data();
            let mut local_output = frame.local_output();
            let mut body = erase_scope_lifetime(ty.as_value());

            for pidx in (0..params.len()).rev() {
                let param = params.get(&target, pidx);
                let param = param.unwrap_unchecked().as_value();
                if param.is::<TypeVar>() {
                    let tvar = param.cast_unchecked::<TypeVar>();
                    let b = UnionAll::new_unchecked(&mut local_output, tvar, body).as_value();
                    body = erase_scope_lifetime(b);
                }
            }

            body.root(target)
        })
    }
}

impl<'base> UnionAll<'base> {
    /// The `UnionAll` `Type`.
    #[inline]
    pub fn type_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_type_type), Private) }
    }

    /// `Type{Tgt} where Tgt<:Tuple`
    #[inline]
    pub fn anytuple_type_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_anytuple_type_type), Private) }
    }

    #[julia_version(until = "1.6")]
    /// The `UnionAll` `Vararg`.
    #[inline]
    pub fn vararg_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_vararg_type), Private) }
    }

    /// The `UnionAll` `AbstractArray`.
    #[inline]
    pub fn abstractarray_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_abstractarray_type), Private) }
    }

    /// The `UnionAll` `DenseArray`.
    #[inline]
    pub fn densearray_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_densearray_type), Private) }
    }

    /// The `UnionAll` `Array`.
    #[inline]
    pub fn array_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_array_type), Private) }
    }

    /// The `UnionAll` `Ptr`.
    #[inline]
    pub fn pointer_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_pointer_type), Private) }
    }

    /// The `UnionAll` `LLVMPtr`.
    #[inline]
    pub fn llvmpointer_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_llvmpointer_type), Private) }
    }

    /// The `UnionAll` `Ref`.
    #[inline]
    pub fn ref_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_ref_type), Private) }
    }

    /// The `UnionAll` `NamedTuple`.
    #[inline]
    pub fn namedtuple_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_namedtuple_type), Private) }
    }

    #[julia_version(since = "1.11")]
    /// The `UnionAll` `GenericMemory`.
    #[inline]
    pub fn genericmemory_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'base>,
    {
        // Safety: global constant
        unsafe {
            Self::wrap_non_null(
                NonNull::new_unchecked(jl_sys::jl_genericmemory_type),
                Private,
            )
        }
    }

    #[julia_version(since = "1.11")]
    /// The `UnionAll` `GenericMemoryRef`.
    #[inline]
    pub fn genericmemoryref_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'base>,
    {
        // Safety: global constant
        unsafe {
            Self::wrap_non_null(
                NonNull::new_unchecked(jl_sys::jl_genericmemoryref_type),
                Private,
            )
        }
    }
}

impl_julia_typecheck!(UnionAll<'scope>, jl_unionall_type, 'scope);
impl_debug!(UnionAll<'_>);

impl<'scope> ManagedPriv<'scope, '_> for UnionAll<'scope> {
    type Wraps = jl_unionall_t;
    type WithLifetimes<'target, 'da> = UnionAll<'target>;
    const NAME: &'static str = "UnionAll";

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

impl_construct_type_managed!(UnionAll, 1, jl_unionall_type);

/// A reference to a [`UnionAll`] that has not been explicitly rooted.
pub type UnionAllRef<'scope> = Ref<'scope, 'static, UnionAll<'scope>>;

/// A [`UnionAllRef`] with static lifetimes. This is a useful shorthand for signatures of
/// `ccall`able functions that return a [`UnionAll`].
pub type UnionAllRet = Ref<'static, 'static, UnionAll<'static>>;

impl_valid_layout!(UnionAllRef, UnionAll, jl_unionall_type);

use crate::memory::target::TargetType;

/// `UnionAll` or `UnionAllRef`, depending on the target type `Tgt`.
pub type UnionAllData<'target, Tgt> =
    <Tgt as TargetType<'target>>::Data<'static, UnionAll<'target>>;

/// `JuliaResult<UnionAll>` or `JuliaResultRef<UnionAllRef>`, depending on the target type `Tgt`.
pub type UnionAllResult<'target, Tgt> = TargetResult<'target, 'static, UnionAll<'target>, Tgt>;

impl_ccall_arg_managed!(UnionAll, 1);
impl_into_typed!(UnionAll);
