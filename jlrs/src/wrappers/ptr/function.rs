//! Wrapper for `Function`, the supertype of all Julia functions.
//!
//! All Julia functions are subtypes of `Function`, a function can be called with the methods
//! of the [`Call`] trait. You don't need to cast a [`Value`] to a [`Function`] in order to call
//! it because [`Value`] also implements [`Call`].
//!
//! [`Call`]: crate::call::Call

use crate::{
    call::{Call, ProvideKeywords, WithKeywords},
    error::JlrsResult,
    layout::{typecheck::Typecheck, valid_layout::ValidLayout},
    memory::{target::global::Global, target::Target},
    private::Private,
    wrappers::ptr::{datatype::DataType, private::WrapperPriv, value::Value, Wrapper},
};
use jl_sys::jl_value_t;
use std::{marker::PhantomData, ptr::NonNull};

use super::{value::ValueResult, Ref};

/// A Julia function.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Function<'scope, 'data> {
    inner: NonNull<jl_value_t>,
    _scope: PhantomData<&'scope ()>,
    _data: PhantomData<&'data ()>,
}

impl<'scope, 'data> Function<'scope, 'data> {
    /// Returns the `DataType` of this function. In Julia, every function has its own `DataType`.
    pub fn datatype(self) -> DataType<'scope> {
        self.as_value().datatype()
    }

    /// Use the target to reroot this data.
    pub fn root<'target, T>(self, target: T) -> FunctionData<'target, 'data, T>
    where
        T: Target<'target>,
    {
        // Safety: the data is valid.
        unsafe { target.data_from_ptr(self.unwrap_non_null(Private), Private) }
    }
}

// Safety: The trait is implemented correctly by using the implementation
// of ValidLayout for FunctionRef
unsafe impl Typecheck for Function<'_, '_> {
    fn typecheck(ty: DataType) -> bool {
        <FunctionRef as ValidLayout>::valid_layout(ty.as_value())
    }
}

impl_debug!(Function<'_, '_>);

impl<'scope, 'data> WrapperPriv<'scope, 'data> for Function<'scope, 'data> {
    type Wraps = jl_value_t;
    type TypeConstructorPriv<'target, 'da> = Function<'target, 'da>;
    const NAME: &'static str = "Function";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self {
            inner,
            _scope: PhantomData,
            _data: PhantomData,
        }
    }

    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.inner
    }
}

impl<'data> Call<'data> for Function<'_, 'data> {
    unsafe fn call0<'target, T>(self, target: T) -> ValueResult<'target, 'data, T>
    where
        T: Target<'target>,
    {
        self.as_value().call0(target)
    }

    unsafe fn call1<'target, T>(
        self,
        target: T,
        arg0: Value<'_, 'data>,
    ) -> ValueResult<'target, 'data, T>
    where
        T: Target<'target>,
    {
        self.as_value().call1(target, arg0)
    }

    unsafe fn call2<'target, T>(
        self,
        target: T,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> ValueResult<'target, 'data, T>
    where
        T: Target<'target>,
    {
        self.as_value().call2(target, arg0, arg1)
    }

    unsafe fn call3<'target, T>(
        self,
        target: T,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
        arg2: Value<'_, 'data>,
    ) -> ValueResult<'target, 'data, T>
    where
        T: Target<'target>,
    {
        self.as_value().call3(target, arg0, arg1, arg2)
    }

    unsafe fn call<'target, 'value, V, T>(
        self,
        target: T,
        args: V,
    ) -> ValueResult<'target, 'data, T>
    where
        V: AsRef<[Value<'value, 'data>]>,
        T: Target<'target>,
    {
        self.as_value().call(target, args)
    }
}

impl<'value, 'data> ProvideKeywords<'value, 'data> for Function<'value, 'data> {
    fn provide_keywords(
        self,
        kws: Value<'value, 'data>,
    ) -> JlrsResult<WithKeywords<'value, 'data>> {
        self.as_value().provide_keywords(kws)
    }
}

impl_root!(Function, 2);

/// A reference to an [`Function`] that has not been explicitly rooted.
pub type FunctionRef<'scope, 'data> = Ref<'scope, 'data, Function<'scope, 'data>>;

// Safety: FunctionRef is valid for ty if ty is a subtype of Function
unsafe impl ValidLayout for FunctionRef<'_, '_> {
    fn valid_layout(ty: Value) -> bool {
        let global = unsafe { Global::new() };
        let function_type = DataType::function_type(&global);
        ty.subtype(function_type.as_value())
    }

    const IS_REF: bool = true;
}

impl_ref_root!(Function, FunctionRef, 2);

use crate::memory::target::target_type::TargetType;

/// `Function` or `FunctionRef`, depending on the target type `T`.
pub type FunctionData<'target, 'data, T> =
    <T as TargetType<'target>>::Data<'data, Function<'target, 'data>>;

/// `JuliaResult<Function>` or `JuliaResultRef<FunctionRef>`, depending on the target type `T`.
pub type FunctionResult<'target, 'data, T> =
    <T as TargetType<'target>>::Result<'data, Function<'target, 'data>>;
