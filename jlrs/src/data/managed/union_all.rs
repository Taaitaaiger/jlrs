//! Managed type for `UnionAll`, A union of types over all values of a type parameter.

use std::{marker::PhantomData, ptr::NonNull};

#[julia_version(since = "1.7")]
use jl_sys::jl_opaque_closure_type;
#[julia_version(until = "1.6")]
use jl_sys::jl_vararg_type;
use jl_sys::{
    jl_abstractarray_type, jl_anytuple_type_type, jl_apply_type, jl_array_type, jl_densearray_type,
    jl_llvmpointer_type, jl_namedtuple_type, jl_pointer_type, jl_ref_type, jl_type_type,
    jl_type_unionall, jl_unionall_t, jl_unionall_type, jl_value_t,
};
use jlrs_macros::julia_version;

use super::{
    erase_scope_lifetime,
    value::{ValueData, ValueResult},
    Managed, Ref,
};
use crate::{
    catch::catch_exceptions,
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
    pub fn new<'target, T>(
        target: T,
        tvar: TypeVar,
        body: Value<'_, 'static>,
    ) -> ValueResult<'target, 'static, T>
    where
        T: Target<'target>,
    {
        // Safety: if an exception is thrown it's caught, the result is immediately rooted
        unsafe {
            let callback = || jl_type_unionall(tvar.unwrap(Private), body.unwrap(Private));
            let exc = |err: Value| err.unwrap_non_null(Private);

            let res = match catch_exceptions(callback, exc) {
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
    pub unsafe fn new_unchecked<'target, T>(
        target: T,
        tvar: TypeVar,
        body: Value<'_, 'static>,
    ) -> ValueData<'target, 'static, T>
    where
        T: Target<'target>,
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

    /*
    inspect(UnionAll):

    var: TypeVar (const)
    body: Any (const)
    */

    /// The body of this `UnionAll`. This is either another `UnionAll` or a `DataType`.
    #[inline]
    pub fn body(self) -> Value<'scope, 'static> {
        // Safety: pointer points to valid data
        unsafe {
            let body = self.unwrap_non_null(Private).as_ref().body;
            debug_assert!(!body.is_null());
            Value::wrap_non_null(NonNull::new_unchecked(body), Private)
        }
    }

    /// The type variable associated with this "layer" of the `UnionAll`.
    #[inline]
    pub fn var(self) -> TypeVar<'scope> {
        // Safety: pointer points to valid data
        unsafe {
            let var = self.unwrap_non_null(Private).as_ref().var;
            debug_assert!(!var.is_null());
            TypeVar::wrap_non_null(NonNull::new_unchecked(var), Private)
        }
    }

    pub unsafe fn apply_types<'target, 'params, V, T>(
        self,
        target: T,
        types: V,
    ) -> ValueResult<'target, 'static, T>
    where
        V: AsRef<[Value<'params, 'static>]>,
        T: Target<'target>,
    {
        let types = types.as_ref();
        let n = types.len();
        let types_ptr = types.as_ptr() as *mut *mut jl_value_t;
        unsafe {
            let callback = || jl_apply_type(self.as_value().unwrap(Private), types_ptr, n);
            let exc = |err: Value| err.unwrap_non_null(Private);

            let res = match catch_exceptions(callback, exc) {
                Ok(ptr) => Ok(NonNull::new_unchecked(ptr)),
                Err(e) => Err(e),
            };

            target.result_from_ptr(res, Private)
        }
    }

    #[inline]
    pub unsafe fn apply_types_unchecked<'target, 'params, V, T>(
        self,
        target: T,
        types: V,
    ) -> ValueData<'target, 'static, T>
    where
        V: AsRef<[Value<'params, 'static>]>,
        T: Target<'target>,
    {
        let types = types.as_ref();
        let n = types.len();
        let types_ptr = types.as_ptr() as *mut *mut jl_value_t;
        let applied = jl_apply_type(self.as_value().unwrap(Private), types_ptr, n);
        debug_assert!(!applied.is_null());
        target.data_from_ptr(NonNull::new_unchecked(applied), Private)
    }

    // TODO: unsafe, document, test
    pub fn rewrap<'target, Tgt: Target<'target>>(
        target: Tgt,
        ty: DataType,
    ) -> ValueData<'target, 'static, Tgt> {
        //
        target
            .with_local_scope::<_, _, 1>(|target, mut frame| unsafe {
                let params = ty.parameters();
                let params = params.data().as_slice();
                let mut local_output = frame.local_output();
                let mut body = erase_scope_lifetime(ty.as_value());

                for param in params.iter().rev().copied() {
                    let param = param.unwrap_unchecked().as_value();
                    if param.is::<TypeVar>() {
                        let tvar = param.cast_unchecked::<TypeVar>();
                        let b = UnionAll::new_unchecked(&mut local_output, tvar, body).as_value();
                        body = erase_scope_lifetime(b);
                    }
                }

                Ok(body.root(target))
            })
            .unwrap()
    }
}

// TODO: use in abstract_types
impl<'base> UnionAll<'base> {
    /// The `UnionAll` `Type`.
    #[inline]
    pub fn type_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_type_type), Private) }
    }

    /// `Type{T} where T<:Tuple`
    #[inline]
    pub fn anytuple_type_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_anytuple_type_type), Private) }
    }

    #[julia_version(until = "1.6")]
    /// The `UnionAll` `Vararg`.
    #[inline]
    pub fn vararg_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_vararg_type), Private) }
    }

    /// The `UnionAll` `AbstractArray`.
    #[inline]
    pub fn abstractarray_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_abstractarray_type), Private) }
    }

    #[julia_version(since = "1.7")]
    /// The `UnionAll` `OpaqueClosure`.
    #[inline]
    pub fn opaque_closure_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_opaque_closure_type), Private) }
    }

    /// The `UnionAll` `DenseArray`.
    #[inline]
    pub fn densearray_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_densearray_type), Private) }
    }

    /// The `UnionAll` `Array`.
    #[inline]
    pub fn array_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_array_type), Private) }
    }

    /// The `UnionAll` `Ptr`.
    #[inline]
    pub fn pointer_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_pointer_type), Private) }
    }

    /// The `UnionAll` `LLVMPtr`.
    #[inline]
    pub fn llvmpointer_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_llvmpointer_type), Private) }
    }

    /// The `UnionAll` `Ref`.
    #[inline]
    pub fn ref_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_ref_type), Private) }
    }

    /// The `UnionAll` `NamedTuple`.
    #[inline]
    pub fn namedtuple_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_namedtuple_type), Private) }
    }
}

impl_julia_typecheck!(UnionAll<'scope>, jl_unionall_type, 'scope);
impl_debug!(UnionAll<'_>);

impl<'scope> ManagedPriv<'scope, '_> for UnionAll<'scope> {
    type Wraps = jl_unionall_t;
    type TypeConstructorPriv<'target, 'da> = UnionAll<'target>;
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

/// `UnionAll` or `UnionAllRef`, depending on the target type `T`.
pub type UnionAllData<'target, T> = <T as TargetType<'target>>::Data<'static, UnionAll<'target>>;

/// `JuliaResult<UnionAll>` or `JuliaResultRef<UnionAllRef>`, depending on the target type `T`.
pub type UnionAllResult<'target, T> = TargetResult<'target, 'static, UnionAll<'target>, T>;

impl_ccall_arg_managed!(UnionAll, 1);
impl_into_typed!(UnionAll);
