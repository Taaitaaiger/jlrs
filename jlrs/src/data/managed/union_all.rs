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

#[julia_version(windows_lts = false)]
use super::value::ValueResult;
use super::{value::ValueData, Managed, Ref};
use crate::{
    data::managed::{datatype::DataType, private::ManagedPriv, type_var::TypeVar, value::Value},
    impl_julia_typecheck,
    memory::target::{ExtendedTarget, Target},
    private::Private,
};

/// An iterated union of types. If a struct field has a parametric type with some of its
/// parameters unknown, its type is represented by a `UnionAll`.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct UnionAll<'scope>(NonNull<jl_unionall_t>, PhantomData<&'scope ()>);

impl<'scope> UnionAll<'scope> {
    #[julia_version(windows_lts = false)]
    /// Create a new `UnionAll`. If an exception is thrown, it's caught and returned.
    pub fn new<'target, T>(
        target: T,
        tvar: TypeVar,
        body: Value<'_, 'static>,
    ) -> ValueResult<'target, 'static, T>
    where
        T: Target<'target>,
    {
        use std::mem::MaybeUninit;

        use jl_sys::jl_value_t;

        use crate::catch::catch_exceptions;

        // Safety: if an exception is thrown it's caught, the result is immediately rooted
        unsafe {
            let mut callback = |result: &mut MaybeUninit<*mut jl_value_t>| {
                let res = jl_type_unionall(tvar.unwrap(Private), body.unwrap(Private));
                result.write(res);
                Ok(())
            };

            let res = match catch_exceptions(&mut callback).unwrap() {
                Ok(ptr) => Ok(NonNull::new_unchecked(ptr)),
                Err(e) => Err(e.ptr()),
            };

            target.result_from_ptr(res, Private)
        }
    }

    /// Create a new `UnionAll`. If an exception is thrown it isn't caught
    ///
    /// Safety: an exception must not be thrown if this method is called from a `ccall`ed
    /// function.
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
    pub fn base_type(self) -> DataType<'scope> {
        let mut b = self;

        // Safety: pointer points to valid data
        while let Ok(body_ua) = b.body().cast::<UnionAll>() {
            b = body_ua;
        }

        // Safety: type at the base must be a DataType
        b.body().cast::<DataType>().unwrap()
    }

    /*
    inspect(UnionAll):

    var: TypeVar (const)
    body: Any (const)
    */

    /// The body of this `UnionAll`. This is either another `UnionAll` or a `DataType`.
    pub fn body(self) -> Value<'scope, 'static> {
        // Safety: pointer points to valid data
        unsafe {
            let body = self.unwrap_non_null(Private).as_ref().body;
            debug_assert!(!body.is_null());
            Value::wrap_non_null(NonNull::new_unchecked(body), Private)
        }
    }

    /// The type variable associated with this "layer" of the `UnionAll`.
    pub fn var(self) -> TypeVar<'scope> {
        // Safety: pointer points to valid data
        unsafe {
            let var = self.unwrap_non_null(Private).as_ref().var;
            debug_assert!(!var.is_null());
            TypeVar::wrap_non_null(NonNull::new_unchecked(var), Private)
        }
    }

    #[julia_version(windows_lts = false)]
    pub unsafe fn apply_types<'target, 'params, V, T>(
        self,
        target: T,
        types: V,
    ) -> ValueResult<'target, 'static, T>
    where
        V: AsRef<[Value<'params, 'static>]>,
        T: Target<'target>,
    {
        use std::mem::MaybeUninit;

        use crate::catch::catch_exceptions;

        let types = types.as_ref();
        let n = types.len();
        let types_ptr = types.as_ptr() as *mut *mut jl_value_t;
        unsafe {
            let mut callback = |result: &mut MaybeUninit<*mut jl_value_t>| {
                let v = jl_apply_type(self.as_value().unwrap(Private), types_ptr, n);

                result.write(v);
                Ok(())
            };

            let res = match catch_exceptions(&mut callback).unwrap() {
                Ok(ptr) => Ok(NonNull::new_unchecked(ptr)),
                Err(e) => Err(e.ptr()),
            };

            target.result_from_ptr(res, Private)
        }
    }

    pub fn rewrap<'target, T: Target<'target>>(
        target: ExtendedTarget<'target, '_, '_, T>,
        ty: DataType,
    ) -> ValueData<'target, 'static, T> {
        //
        let (target, frame) = target.split();

        frame
            .scope(|mut frame| {
                let params = ty.parameters();
                let params = params.data().as_slice();
                let mut body = ty.as_value();

                for param in params.iter().copied() {
                    unsafe {
                        let param = param.unwrap().as_value();
                        if let Ok(tvar) = param.cast::<TypeVar>() {
                            body = UnionAll::new_unchecked(&mut frame, tvar, body).as_value();
                        }
                    }
                }

                Ok(body.root(target))
            })
            .unwrap()
    }

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
}

impl<'base> UnionAll<'base> {
    /// The `UnionAll` `Type`.
    pub fn type_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_type_type), Private) }
    }

    /// `Type{T} where T<:Tuple`
    pub fn anytuple_type_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_anytuple_type_type), Private) }
    }

    #[julia_version(until = "1.6")]
    /// The `UnionAll` `Vararg`.
    pub fn vararg_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_vararg_type), Private) }
    }

    /// The `UnionAll` `AbstractArray`.
    pub fn abstractarray_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_abstractarray_type), Private) }
    }

    #[julia_version(since = "1.7")]
    /// The `UnionAll` `OpaqueClosure`.
    pub fn opaque_closure_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_opaque_closure_type), Private) }
    }

    /// The `UnionAll` `DenseArray`.
    pub fn densearray_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_densearray_type), Private) }
    }

    /// The `UnionAll` `Array`.
    pub fn array_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_array_type), Private) }
    }

    /// The `UnionAll` `Ptr`.
    pub fn pointer_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_pointer_type), Private) }
    }

    /// The `UnionAll` `LLVMPtr`.
    pub fn llvmpointer_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_llvmpointer_type), Private) }
    }

    /// The `UnionAll` `Ref`.
    pub fn ref_type<T>(_: &T) -> Self
    where
        T: Target<'base>,
    {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_ref_type), Private) }
    }

    /// The `UnionAll` `NamedTuple`.
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
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

impl_construct_type_managed!(UnionAll<'_>, jl_unionall_type);

/// A reference to a [`UnionAll`] that has not been explicitly rooted.
pub type UnionAllRef<'scope> = Ref<'scope, 'static, UnionAll<'scope>>;

/// A [`UnionAllRef`] with static lifetimes. This is a useful shorthand for signatures of
/// `ccall`able functions that return a [`UnionAll`].
pub type UnionAllRet = Ref<'static, 'static, UnionAll<'static>>;

impl_valid_layout!(UnionAllRef, UnionAll);

use crate::memory::target::target_type::TargetType;

/// `UnionAll` or `UnionAllRef`, depending on the target type `T`.
pub type UnionAllData<'target, T> = <T as TargetType<'target>>::Data<'static, UnionAll<'target>>;

/// `JuliaResult<UnionAll>` or `JuliaResultRef<UnionAllRef>`, depending on the target type `T`.
pub type UnionAllResult<'target, T> =
    <T as TargetType<'target>>::Result<'static, UnionAll<'target>>;

impl_ccall_arg_managed!(UnionAll, 1);
impl_into_typed!(UnionAll);
